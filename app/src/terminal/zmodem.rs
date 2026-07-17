use std::{
    collections::VecDeque,
    ffi::OsString,
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

use tempfile::NamedTempFile;
use zmodem2::{Action, Event, FileInfo, Position};

const ZMODEM_HEX_HEADER_PREFIX: &[u8] = b"**\x18B";
const ZMODEM_UPLOAD_FRAME: &[u8] = b"01";
const ZMODEM_DOWNLOAD_FRAME: &[u8] = b"00";
pub(crate) const ZMODEM_ABORT_SEQUENCE: &[u8] = b"\x18\x18\x18\x18\x18\x08\x08\x08\x08\x08";
const ZMODEM_HEX_HEADER_PAYLOAD_LEN: usize = 14;
const ZMODEM_DETECTION_WINDOW_LEN: usize =
    ZMODEM_HEX_HEADER_PREFIX.len() + ZMODEM_HEX_HEADER_PAYLOAD_LEN;
const ZMODEM_ZDLE: u8 = 0x18;
const ZMODEM_XON: u8 = 0x11;
const ZMODEM_ZCRCW: u8 = 0x6b;
const ZMODEM_ZRINIT_FRAME: u8 = 0x01;
const ZMODEM_ZRINIT_CANOVIO: u8 = 0x02;
const ZMODEM_ZRINIT_CANFC32: u8 = 0x20;
const ZMODEM_FAST_WINDOW_SIZE: u16 = u16::MAX;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZmodemDirection {
    /// 本地文件发送给终端里的程序，通常是远端 `rz`。
    Upload,
    /// 终端里的程序发送文件，通常是远端 `sz`，本地负责保存。
    Download,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZmodemTransferPaths {
    pub direction: ZmodemDirection,
    pub paths: Vec<PathBuf>,
}

impl ZmodemTransferPaths {
    pub fn upload(paths: Vec<PathBuf>) -> Self {
        Self {
            direction: ZmodemDirection::Upload,
            paths,
        }
    }

    pub fn download_directory(path: PathBuf) -> Self {
        Self {
            direction: ZmodemDirection::Download,
            paths: vec![path],
        }
    }

    pub fn cancel(direction: ZmodemDirection) -> Self {
        Self {
            direction,
            paths: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ZmodemEvent {
    UploadRequested,
    DownloadDirectoryRequested,
    AbortInteractiveReceiver,
    Started {
        direction: ZmodemDirection,
    },
    FileStarted {
        direction: ZmodemDirection,
        name: String,
        size: Option<u64>,
        path: Option<PathBuf>,
    },
    Progress {
        direction: ZmodemDirection,
        name: String,
        transferred: u64,
        total: Option<u64>,
    },
    FileCompleted {
        direction: ZmodemDirection,
        name: String,
        path: Option<PathBuf>,
    },
    Completed {
        direction: ZmodemDirection,
    },
    Cancelled {
        direction: ZmodemDirection,
    },
    Failed {
        direction: Option<ZmodemDirection>,
        message: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ZmodemDetection {
    pub direction: ZmodemDirection,
    pub start_index: usize,
}

pub fn detect_zmodem_session(bytes: &[u8]) -> Option<ZmodemDetection> {
    bytes
        .windows(ZMODEM_DETECTION_WINDOW_LEN)
        .enumerate()
        .find_map(|(start_index, window)| {
            if !window.starts_with(ZMODEM_HEX_HEADER_PREFIX) {
                return None;
            }
            let frame_start = ZMODEM_HEX_HEADER_PREFIX.len();
            let header_payload =
                &bytes[start_index + frame_start..start_index + ZMODEM_DETECTION_WINDOW_LEN];
            if !header_payload.iter().all(u8::is_ascii_hexdigit) {
                return None;
            }

            let frame = &header_payload[..2];
            let direction = if frame == ZMODEM_UPLOAD_FRAME {
                ZmodemDirection::Upload
            } else if frame == ZMODEM_DOWNLOAD_FRAME {
                ZmodemDirection::Download
            } else {
                return None;
            };
            Some(ZmodemDetection {
                direction,
                start_index,
            })
        })
}

#[derive(Debug, Default)]
pub struct ZmodemDetector {
    pending: Vec<u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ZmodemDetectorResult {
    Ordinary(Vec<u8>),
    Detected {
        detection: ZmodemDetection,
        ordinary_output: Vec<u8>,
        zmodem_input: Vec<u8>,
    },
}

impl ZmodemDetector {
    pub fn push(&mut self, bytes: &[u8]) -> ZmodemDetectorResult {
        self.pending.extend_from_slice(bytes);

        if let Some(detection) = detect_zmodem_session(&self.pending) {
            let ordinary_output = self.pending[..detection.start_index].to_vec();
            let zmodem_input = self.pending[detection.start_index..].to_vec();
            self.pending.clear();
            return ZmodemDetectorResult::Detected {
                detection: ZmodemDetection {
                    direction: detection.direction,
                    start_index: 0,
                },
                ordinary_output,
                zmodem_input,
            };
        }

        let keep_len = longest_zmodem_prefix_suffix_len(&self.pending);
        let ordinary_len = self.pending.len().saturating_sub(keep_len);
        if ordinary_len == 0 {
            return ZmodemDetectorResult::Ordinary(Vec::new());
        }

        let ordinary_output = self.pending.drain(..ordinary_len).collect();
        ZmodemDetectorResult::Ordinary(ordinary_output)
    }

    pub fn take_pending_ordinary(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.pending)
    }
}

fn longest_zmodem_prefix_suffix_len(bytes: &[u8]) -> usize {
    let max_len = bytes.len().min(ZMODEM_DETECTION_WINDOW_LEN - 1);
    for len in (1..=max_len).rev() {
        let suffix = &bytes[bytes.len() - len..];
        if ZMODEM_HEX_HEADER_PREFIX.starts_with(suffix) {
            return len;
        }
        if suffix.starts_with(ZMODEM_HEX_HEADER_PREFIX) {
            return len;
        }
    }
    0
}

pub struct PendingZmodemSession {
    direction: ZmodemDirection,
    input: Vec<u8>,
}

impl PendingZmodemSession {
    pub fn new(direction: ZmodemDirection, initial_input: &[u8]) -> Self {
        Self {
            direction,
            input: initial_input.to_vec(),
        }
    }

    pub fn direction(&self) -> ZmodemDirection {
        self.direction
    }

    pub fn append_input(&mut self, bytes: &[u8]) {
        self.input.extend_from_slice(bytes);
    }

    pub fn cancel(self) -> Vec<u8> {
        ZMODEM_ABORT_SEQUENCE.to_vec()
    }

    pub fn start(self, paths: ZmodemTransferPaths) -> anyhow::Result<ZmodemSession> {
        if paths.direction != self.direction {
            anyhow::bail!(
                "selected paths for {:?}, but pending ZMODEM session is {:?}",
                paths.direction,
                self.direction
            );
        }

        match self.direction {
            ZmodemDirection::Upload => {
                let mut session = UploadSession::new_after_receiver_init(paths.paths)?;
                session
                    .input
                    .extend(normalize_remote_zrinit_for_upload(&self.input));
                Ok(ZmodemSession::Upload(session))
            }
            ZmodemDirection::Download => {
                let dest_dir = paths
                    .paths
                    .into_iter()
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("no destination directory selected"))?;
                let mut session = DownloadSession::new(dest_dir)?;
                session.input.extend(self.input);
                Ok(ZmodemSession::Download(session))
            }
        }
    }
}

pub enum ZmodemSession {
    Upload(UploadSession),
    Download(DownloadSession),
    #[cfg(test)]
    Mock(MockZmodemSession),
}

impl ZmodemSession {
    pub fn upload_after_receiver_init(paths: Vec<PathBuf>) -> anyhow::Result<Self> {
        Ok(Self::Upload(UploadSession::new_after_receiver_init(paths)?))
    }

    pub fn download_to_directory(dest_dir: PathBuf) -> anyhow::Result<Self> {
        Ok(Self::Download(DownloadSession::new(dest_dir)?))
    }

    pub fn direction(&self) -> ZmodemDirection {
        match self {
            Self::Upload(_) => ZmodemDirection::Upload,
            Self::Download(_) => ZmodemDirection::Download,
            #[cfg(test)]
            Self::Mock(session) => session.direction(),
        }
    }

    pub fn append_input(&mut self, bytes: &[u8]) {
        match self {
            Self::Upload(session) => {
                session.input.extend(bytes);
                normalize_pending_upload_input(&mut session.input);
            }
            Self::Download(session) => session.input.extend(bytes),
            #[cfg(test)]
            Self::Mock(session) => session.append_input(bytes),
        }
    }

    pub fn take_input(&mut self) -> Vec<u8> {
        match self {
            Self::Upload(session) => drain_vec_deque(&mut session.input),
            Self::Download(session) => drain_vec_deque(&mut session.input),
            #[cfg(test)]
            Self::Mock(session) => session.take_input(),
        }
    }

    pub fn cancel(mut self) -> Vec<u8> {
        match &mut self {
            Self::Upload(session) => session.sender.abort(),
            Self::Download(session) => {
                let _ = session.receiver.abort();
            }
            #[cfg(test)]
            Self::Mock(_) => {}
        }
        ZMODEM_ABORT_SEQUENCE.to_vec()
    }

    pub fn drain_actions(
        &mut self,
        mut queue_wire: impl FnMut(Vec<u8>),
        mut emit_event: impl FnMut(ZmodemEvent),
    ) -> anyhow::Result<bool> {
        match self {
            Self::Upload(session) => session.drain_actions(&mut queue_wire, &mut emit_event),
            Self::Download(session) => session.drain_actions(&mut queue_wire, &mut emit_event),
            #[cfg(test)]
            Self::Mock(session) => session.drain_actions(queue_wire, emit_event),
        }
    }
}

#[cfg(test)]
pub struct MockZmodemSession {
    pub direction: ZmodemDirection,
    pub input: Vec<u8>,
    pub events: Vec<ZmodemEvent>,
    pub done: bool,
}

#[cfg(test)]
impl MockZmodemSession {
    fn direction(&self) -> ZmodemDirection {
        self.direction
    }

    fn append_input(&mut self, bytes: &[u8]) {
        self.input.extend_from_slice(bytes);
    }

    fn take_input(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.input)
    }

    fn drain_actions(
        &mut self,
        _queue_wire: impl FnMut(Vec<u8>),
        mut emit_event: impl FnMut(ZmodemEvent),
    ) -> anyhow::Result<bool> {
        for event in self.events.drain(..) {
            emit_event(event);
        }
        Ok(self.done)
    }
}

pub struct UploadSession {
    sender: zmodem2::Sender,
    input: VecDeque<u8>,
    files: Vec<UploadFile>,
    current_index: usize,
    current_file: Option<File>,
    current_file_started: bool,
    started: bool,
    completed: bool,
}

struct UploadFile {
    path: PathBuf,
    wire_name: Vec<u8>,
    display_name: String,
    size: u64,
}

impl UploadSession {
    fn new(paths: Vec<PathBuf>) -> anyhow::Result<Self> {
        Self::new_with_options(paths, false)
    }

    fn new_after_receiver_init(paths: Vec<PathBuf>) -> anyhow::Result<Self> {
        Self::new_with_options(paths, true)
    }

    fn new_with_options(
        paths: Vec<PathBuf>,
        acknowledge_initial_zrqinit: bool,
    ) -> anyhow::Result<Self> {
        if paths.is_empty() {
            anyhow::bail!("no files selected for upload");
        }

        let files = paths
            .into_iter()
            .map(UploadFile::from_path)
            .collect::<anyhow::Result<Vec<_>>>()?;

        let mut sender = zmodem2::Sender::new()?;
        if acknowledge_initial_zrqinit {
            acknowledge_pending_sender_wire(&mut sender);
        }

        let mut session = Self {
            sender,
            input: VecDeque::new(),
            files,
            current_index: 0,
            current_file: None,
            current_file_started: false,
            started: false,
            completed: false,
        };
        session.start_current_file()?;
        Ok(session)
    }

    fn start_current_file(&mut self) -> anyhow::Result<()> {
        let Some(file) = self.files.get(self.current_index) else {
            self.sender.finish()?;
            return Ok(());
        };

        let size = u32::try_from(file.size).map_err(|_| {
            anyhow::anyhow!(
                "ZMODEM upload only supports files up to {} bytes; {} is {} bytes",
                u32::MAX,
                file.path.display(),
                file.size
            )
        })?;
        self.current_file = Some(File::open(&file.path)?);
        self.current_file_started = false;
        self.sender
            .start_file(FileInfo::new(&file.wire_name, Some(Position::new(size))))?;
        Ok(())
    }

    fn current_file_info(&self) -> anyhow::Result<&UploadFile> {
        self.files
            .get(self.current_index)
            .ok_or_else(|| anyhow::anyhow!("ZMODEM upload has no current file"))
    }

    fn emit_current_file_started(
        &mut self,
        emit_event: &mut impl FnMut(ZmodemEvent),
    ) -> anyhow::Result<()> {
        if self.current_file_started {
            return Ok(());
        }

        let (display_name, size, path) = {
            let file = self.current_file_info()?;
            (file.display_name.clone(), file.size, file.path.clone())
        };
        self.current_file_started = true;
        emit_event(ZmodemEvent::FileStarted {
            direction: ZmodemDirection::Upload,
            name: display_name,
            size: Some(size),
            path: Some(path),
        });
        Ok(())
    }

    fn drain_actions(
        &mut self,
        queue_wire: &mut impl FnMut(Vec<u8>),
        emit_event: &mut impl FnMut(ZmodemEvent),
    ) -> anyhow::Result<bool> {
        if !self.started {
            self.started = true;
            emit_event(ZmodemEvent::Started {
                direction: ZmodemDirection::Upload,
            });
        }

        loop {
            match self.sender.poll() {
                Action::WriteWire(bytes) => {
                    let len = bytes.len();
                    let wire = prepare_zmodem_wire_bytes(bytes);
                    log::debug!(
                        "ZMODEM upload queueing wire bytes: raw_len={len}, queued_len={}",
                        wire.len()
                    );
                    queue_wire(wire);
                    self.sender.wire_written(len);
                }
                Action::ReadFile { offset, max_len } => {
                    self.emit_current_file_started(emit_event)?;
                    let mut buf = vec![0; max_len];
                    let file = self
                        .current_file
                        .as_mut()
                        .ok_or_else(|| anyhow::anyhow!("ZMODEM upload file is not open"))?;
                    file.seek(SeekFrom::Start(u64::from(offset.get())))?;
                    let bytes_read = file.read(&mut buf)?;
                    if bytes_read == 0 {
                        anyhow::bail!("unexpected end of file during ZMODEM upload");
                    }
                    log::debug!(
                        "ZMODEM upload read file chunk: offset={}, max_len={max_len}, bytes_read={bytes_read}",
                        offset.get()
                    );
                    buf.truncate(bytes_read);
                    self.sender.submit_file(&buf)?;

                    let file_info = self.current_file_info()?;
                    emit_event(ZmodemEvent::Progress {
                        direction: ZmodemDirection::Upload,
                        name: file_info.display_name.clone(),
                        transferred: u64::from(offset.get()) + bytes_read as u64,
                        total: Some(file_info.size),
                    });
                }
                Action::Event(event) => match event {
                    Event::FileCompleted => {
                        self.emit_current_file_started(emit_event)?;
                        let file = self.current_file_info()?;
                        emit_event(ZmodemEvent::FileCompleted {
                            direction: ZmodemDirection::Upload,
                            name: file.display_name.clone(),
                            path: Some(file.path.clone()),
                        });
                        self.current_file.take();
                        self.current_index += 1;
                        if self.current_index < self.files.len() {
                            self.start_current_file()?;
                        } else {
                            self.sender.finish()?;
                        }
                    }
                    Event::SessionCompleted => {
                        self.completed = true;
                        emit_event(ZmodemEvent::Completed {
                            direction: ZmodemDirection::Upload,
                        });
                        return Ok(true);
                    }
                    Event::Aborted => {
                        self.completed = true;
                        emit_event(ZmodemEvent::Cancelled {
                            direction: ZmodemDirection::Upload,
                        });
                        return Ok(true);
                    }
                    Event::FileStarted(_) => {}
                    _ => {}
                },
                Action::Idle => {
                    if self.input.is_empty() {
                        return Ok(self.completed);
                    }
                    let input = drain_vec_deque(&mut self.input);
                    let consumed = self.sender.submit_wire(&input)?;
                    log::debug!(
                        "ZMODEM upload consumed receiver wire: input_len={}, consumed={consumed}",
                        input.len()
                    );
                    if consumed < input.len() {
                        self.input.extend(input[consumed..].iter().copied());
                    }
                    if consumed == 0 {
                        return Ok(self.completed);
                    }
                }
                Action::WriteFile(_) => {
                    anyhow::bail!("ZMODEM sender unexpectedly requested file write");
                }
                _ => {}
            }
        }
    }
}

fn acknowledge_pending_sender_wire(sender: &mut zmodem2::Sender) {
    while let Action::WriteWire(bytes) = sender.poll() {
        let len = bytes.len();
        sender.wire_written(len);
    }
}

impl UploadFile {
    fn from_path(path: PathBuf) -> anyhow::Result<Self> {
        let metadata = path.metadata()?;
        if !metadata.is_file() {
            anyhow::bail!("ZMODEM upload only supports files: {}", path.display());
        }
        let file_name = path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("file has no name: {}", path.display()))?
            .to_string_lossy()
            .to_string();
        Ok(Self {
            path,
            wire_name: file_name.as_bytes().to_vec(),
            display_name: file_name,
            size: metadata.len(),
        })
    }
}

pub struct DownloadSession {
    receiver: zmodem2::Receiver,
    input: VecDeque<u8>,
    dest_dir: PathBuf,
    current_file: Option<DownloadFile>,
    completed_files: Vec<CompletedDownloadFile>,
    started: bool,
    completed: bool,
}

struct DownloadFile {
    temp_file: NamedTempFile,
    name: String,
    final_path: PathBuf,
    size: Option<u64>,
    transferred: u64,
}

struct CompletedDownloadFile {
    temp_file: NamedTempFile,
    name: String,
    final_path: PathBuf,
}

impl DownloadSession {
    fn new(dest_dir: PathBuf) -> anyhow::Result<Self> {
        if !dest_dir.is_dir() {
            anyhow::bail!(
                "ZMODEM destination is not a directory: {}",
                dest_dir.display()
            );
        }

        Ok(Self {
            receiver: zmodem2::Receiver::new()?,
            input: VecDeque::new(),
            dest_dir,
            current_file: None,
            completed_files: Vec::new(),
            started: false,
            completed: false,
        })
    }

    fn drain_actions(
        &mut self,
        queue_wire: &mut impl FnMut(Vec<u8>),
        emit_event: &mut impl FnMut(ZmodemEvent),
    ) -> anyhow::Result<bool> {
        if !self.started {
            self.started = true;
            emit_event(ZmodemEvent::Started {
                direction: ZmodemDirection::Download,
            });
        }

        loop {
            match self.receiver.poll() {
                Action::WriteWire(bytes) => {
                    let len = bytes.len();
                    let wire = prepare_zmodem_receiver_wire_bytes(bytes);
                    log::debug!(
                        "ZMODEM download queueing wire bytes: raw_len={len}, queued_len={}",
                        wire.len()
                    );
                    queue_wire(wire);
                    self.receiver.wire_written(len);
                }
                Action::WriteFile(bytes) => {
                    let bytes = bytes.to_vec();
                    let len = bytes.len();
                    let (name, transferred, total) = {
                        let current_file = self
                            .current_file
                            .as_mut()
                            .ok_or_else(|| anyhow::anyhow!("ZMODEM download file is not open"))?;
                        current_file.temp_file.write_all(&bytes)?;
                        current_file.transferred += len as u64;
                        (
                            current_file.name.clone(),
                            current_file.transferred,
                            current_file.size,
                        )
                    };
                    self.receiver.file_written(len)?;
                    emit_event(ZmodemEvent::Progress {
                        direction: ZmodemDirection::Download,
                        name,
                        transferred,
                        total,
                    });
                }
                Action::Event(event) => match event {
                    Event::FileStarted(info) => {
                        let (name, size) = {
                            let name = sanitize_wire_file_name(info.name)?;
                            let size = info.size.map(|size| u64::from(size.get()));
                            (name, size)
                        };
                        let final_path = self.unused_final_path(&name);
                        let temp_file = tempfile::Builder::new()
                            .prefix(".warp-zmodem-")
                            .suffix(".part")
                            .tempfile_in(&self.dest_dir)?;
                        self.current_file = Some(DownloadFile {
                            temp_file,
                            name: name.clone(),
                            final_path: final_path.clone(),
                            size,
                            transferred: 0,
                        });
                        emit_event(ZmodemEvent::FileStarted {
                            direction: ZmodemDirection::Download,
                            name,
                            size,
                            path: Some(final_path),
                        });
                    }
                    Event::FileCompleted => {
                        if let Some(mut file) = self.current_file.take() {
                            file.temp_file.flush()?;
                            self.completed_files.push(CompletedDownloadFile {
                                temp_file: file.temp_file,
                                name: file.name,
                                final_path: file.final_path,
                            });
                        }
                    }
                    Event::SessionCompleted => {
                        self.persist_completed_files(emit_event)?;
                        self.completed = true;
                        emit_event(ZmodemEvent::Completed {
                            direction: ZmodemDirection::Download,
                        });
                        return Ok(true);
                    }
                    Event::Aborted => {
                        self.current_file.take();
                        self.completed = true;
                        emit_event(ZmodemEvent::Cancelled {
                            direction: ZmodemDirection::Download,
                        });
                        return Ok(true);
                    }
                    _ => {}
                },
                Action::Idle => {
                    if self.input.is_empty() {
                        return Ok(self.completed);
                    }
                    let input = drain_vec_deque(&mut self.input);
                    let consumed = self.receiver.submit_wire(&input)?;
                    log::debug!(
                        "ZMODEM download consumed sender wire: input_len={}, consumed={consumed}",
                        input.len()
                    );
                    if consumed < input.len() {
                        self.input.extend(input[consumed..].iter().copied());
                    }
                    if consumed == 0 {
                        return Ok(self.completed);
                    }
                }
                Action::ReadFile { .. } => {
                    anyhow::bail!("ZMODEM receiver unexpectedly requested file read");
                }
                _ => {}
            }
        }
    }

    fn unused_final_path(&self, name: &str) -> PathBuf {
        let mut reserved_paths = self
            .completed_files
            .iter()
            .map(|file| file.final_path.clone())
            .collect::<Vec<_>>();
        if let Some(current_file) = &self.current_file {
            reserved_paths.push(current_file.final_path.clone());
        }
        unused_path_avoiding(&self.dest_dir, name, &reserved_paths)
    }

    fn persist_completed_files(
        &mut self,
        emit_event: &mut impl FnMut(ZmodemEvent),
    ) -> anyhow::Result<()> {
        for file in self.completed_files.drain(..) {
            let final_path = file.final_path.clone();
            file.temp_file
                .persist_noclobber(&final_path)
                .map_err(|err| {
                    anyhow::anyhow!(
                        "failed to save ZMODEM download to {}: {}",
                        final_path.display(),
                        err.error
                    )
                })?;
            log::debug!(
                "ZMODEM download committed completed file {} to {}",
                file.name,
                final_path.display()
            );
            emit_event(ZmodemEvent::FileCompleted {
                direction: ZmodemDirection::Download,
                name: file.name,
                path: Some(final_path),
            });
        }
        Ok(())
    }
}

fn drain_vec_deque(input: &mut VecDeque<u8>) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(input.len());
    while let Some(byte) = input.pop_front() {
        bytes.push(byte);
    }
    bytes
}

fn prepare_zmodem_wire_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut wire = bytes.to_vec();
    if zmodem_wire_has_zcrcw_subpacket(bytes) && wire.last() != Some(&ZMODEM_XON) {
        wire.push(ZMODEM_XON);
    }
    wire
}

fn prepare_zmodem_receiver_wire_bytes(bytes: &[u8]) -> Vec<u8> {
    rewrite_zrinit_wire(bytes).unwrap_or_else(|| prepare_zmodem_wire_bytes(bytes))
}

fn normalize_remote_zrinit_for_upload(bytes: &[u8]) -> Vec<u8> {
    rewrite_zrinit_wire(bytes).unwrap_or_else(|| bytes.to_vec())
}

fn normalize_pending_upload_input(input: &mut VecDeque<u8>) {
    if input.len() < ZMODEM_DETECTION_WINDOW_LEN {
        return;
    }
    let bytes = input.iter().copied().collect::<Vec<_>>();
    let normalized = normalize_remote_zrinit_for_upload(&bytes);
    if normalized != bytes {
        input.clear();
        input.extend(normalized);
    }
}

fn zmodem_wire_has_zcrcw_subpacket(bytes: &[u8]) -> bool {
    bytes
        .windows(2)
        .any(|window| window == [ZMODEM_ZDLE, ZMODEM_ZCRCW])
}

fn rewrite_zrinit_wire(bytes: &[u8]) -> Option<Vec<u8>> {
    if !bytes.starts_with(ZMODEM_HEX_HEADER_PREFIX) {
        return None;
    }

    let header_end = ZMODEM_HEX_HEADER_PREFIX.len() + ZMODEM_HEX_HEADER_PAYLOAD_LEN;
    let header_payload = bytes.get(ZMODEM_HEX_HEADER_PREFIX.len()..header_end)?;
    if !header_payload.iter().all(u8::is_ascii_hexdigit) {
        return None;
    }

    let mut decoded = [0u8; 7];
    decode_zmodem_hex_header_payload(header_payload, &mut decoded)?;
    if decoded[0] != ZMODEM_ZRINIT_FRAME {
        return None;
    }

    let window = ZMODEM_FAST_WINDOW_SIZE.to_le_bytes();
    decoded[1] = window[0];
    decoded[2] = window[1];
    decoded[4] |= ZMODEM_ZRINIT_CANOVIO | ZMODEM_ZRINIT_CANFC32;
    let crc = crc16_xmodem(&decoded[..5]).to_be_bytes();
    decoded[5] = crc[0];
    decoded[6] = crc[1];

    let mut rewritten = Vec::with_capacity(bytes.len());
    rewritten.extend_from_slice(ZMODEM_HEX_HEADER_PREFIX);
    rewritten.extend_from_slice(&encode_zmodem_hex_header_payload(&decoded));
    rewritten.extend_from_slice(&bytes[header_end..]);
    Some(rewritten)
}

fn decode_zmodem_hex_header_payload(hex: &[u8], decoded: &mut [u8; 7]) -> Option<()> {
    for (chunk, byte) in hex.chunks_exact(2).zip(decoded.iter_mut()) {
        let high = decode_zmodem_hex_nibble(chunk[0])?;
        let low = decode_zmodem_hex_nibble(chunk[1])?;
        *byte = (high << 4) | low;
    }
    Some(())
}

fn decode_zmodem_hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn encode_zmodem_hex_header_payload(decoded: &[u8; 7]) -> [u8; ZMODEM_HEX_HEADER_PAYLOAD_LEN] {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = [0u8; ZMODEM_HEX_HEADER_PAYLOAD_LEN];
    for (index, byte) in decoded.iter().enumerate() {
        encoded[index * 2] = HEX[(byte >> 4) as usize];
        encoded[index * 2 + 1] = HEX[(byte & 0x0f) as usize];
    }
    encoded
}

fn crc16_xmodem(data: &[u8]) -> u16 {
    let mut crc = 0u16;
    for byte in data {
        crc = crc16_xmodem_update(crc, *byte);
    }
    crc
}

fn crc16_xmodem_update(mut crc: u16, byte: u8) -> u16 {
    crc ^= (byte as u16) << 8;
    for _ in 0..8 {
        if (crc & 0x8000) != 0 {
            crc = (crc << 1) ^ 0x1021;
        } else {
            crc <<= 1;
        }
    }
    crc
}

pub fn sanitize_wire_file_name(name: &[u8]) -> anyhow::Result<String> {
    let decoded = String::from_utf8_lossy(name);
    let file_name = decoded
        .split(['/', '\\'])
        .filter(|part| !part.is_empty())
        .next_back()
        .ok_or_else(|| anyhow::anyhow!("received ZMODEM file without a name"))?;
    if file_name == "." || file_name == ".." {
        anyhow::bail!("received unsafe ZMODEM file name: {file_name}");
    }
    Ok(file_name.to_owned())
}

pub fn unused_path(dest_dir: &Path, file_name: &str) -> PathBuf {
    unused_path_avoiding(dest_dir, file_name, &[])
}

fn unused_path_avoiding(dest_dir: &Path, file_name: &str, reserved_paths: &[PathBuf]) -> PathBuf {
    let candidate = dest_dir.join(file_name);
    if !candidate.exists() && !reserved_paths.iter().any(|path| path == &candidate) {
        return candidate;
    }

    let path = Path::new(file_name);
    let stem = path
        .file_stem()
        .filter(|stem| !stem.is_empty())
        .unwrap_or_else(|| path.as_os_str());
    let extension = path.extension();

    for index in 1.. {
        let mut next_name = OsString::from(stem);
        next_name.push(format!(" ({index})"));
        if let Some(extension) = extension {
            next_name.push(".");
            next_name.push(extension);
        }
        let candidate = dest_dir.join(next_name);
        if !candidate.exists() && !reserved_paths.iter().any(|path| path == &candidate) {
            return candidate;
        }
    }

    unreachable!("usize iteration should find an unused path")
}

pub fn zmodem_error_event(
    direction: Option<ZmodemDirection>,
    error: impl std::fmt::Display,
) -> ZmodemEvent {
    ZmodemEvent::Failed {
        direction,
        message: error.to_string(),
    }
}

#[cfg(test)]
#[path = "zmodem_tests.rs"]
mod tests;
