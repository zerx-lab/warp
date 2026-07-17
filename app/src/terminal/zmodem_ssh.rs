use std::{
    collections::HashMap,
    ffi::OsString,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{ChildStderr, ChildStdout, ExitStatus, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, RecvTimeoutError},
        Arc, Mutex,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

use anyhow::{anyhow, Context as _};
use zeroize::Zeroizing;

use crate::terminal::ssh::util::InteractiveSshCommand;
use crate::terminal::zmodem::{
    ZmodemDirection, ZmodemEvent, ZmodemSession, ZmodemTransferPaths, ZMODEM_ABORT_SEQUENCE,
};

const SSH_ZMODEM_TIMEOUT: Duration = Duration::from_secs(60 * 60);
const SSH_ZMODEM_INITIAL_OUTPUT_TIMEOUT: Duration = Duration::from_secs(15);
const SSH_ZMODEM_POLL_INTERVAL: Duration = Duration::from_millis(100);
const SSH_ZMODEM_STDERR_LIMIT: usize = 8 * 1024;
const SSH_ZMODEM_RAW_BUFFER_SIZE: usize = 1024 * 1024;
const SSH_ZMODEM_RAW_WRITE_CHUNK_SIZE: usize = 256 * 1024;
const SSH_ZMODEM_PROGRESS_EVENT_INTERVAL: Duration = Duration::from_millis(500);
const SSH_ZMODEM_RAW_DOWNLOAD_MAX_HEADER_LEN: usize = 1024;
const SSH_ZMODEM_RAW_DOWNLOAD_MAX_NAME_LEN: usize = 4096;
const SSH_ZMODEM_RAW_DOWNLOAD_MAX_FILE_SIZE: u64 = 32 * 1024 * 1024 * 1024;
const SSH_ZMODEM_RAW_DOWNLOAD_MAX_TOTAL_SIZE: u64 = 128 * 1024 * 1024 * 1024;
const SSH_ZMODEM_RAW_UPLOAD_READY: &[u8] = b"WARP_ZMODEM_READY\n";
const SSH_ZMODEM_RAW_UPLOAD_STAGED: &[u8] = b"WARP_ZMODEM_STAGED\n";
const SSH_ZMODEM_RAW_UPLOAD_COMMIT: &[u8] = b"WARP_ZMODEM_COMMIT\n";
const SSH_ZMODEM_REMOTE_RZ_TOKEN_ENV: &str = "WARP_ZMODEM_TOKEN";

enum StdoutReadEvent {
    Bytes(Vec<u8>),
    Eof,
    Error(String),
}

struct StderrReader {
    buffer: Arc<Mutex<Vec<u8>>>,
    handle: JoinHandle<Vec<u8>>,
}

impl StderrReader {
    fn snapshot_text(&self) -> Option<String> {
        let bytes = self.buffer.lock().ok()?.clone();
        sanitize_stderr(&bytes)
    }
}

#[derive(Clone, Debug)]
struct RawUploadFileSpec {
    path: PathBuf,
    name: String,
    size: u64,
}

#[derive(Clone, Debug, Default)]
pub struct LegacySshZmodemUploadCancellation {
    inner: Arc<LegacySshZmodemUploadCancellationInner>,
}

#[derive(Debug, Default)]
struct LegacySshZmodemUploadCancellationInner {
    cancelled: AtomicBool,
    child: Mutex<Option<Arc<Mutex<std::process::Child>>>>,
}

impl LegacySshZmodemUploadCancellation {
    pub fn cancel(&self) {
        self.inner.cancelled.store(true, Ordering::Relaxed);
        let child = match self.inner.child.try_lock() {
            Ok(child) => child.as_ref().cloned(),
            Err(_) => None,
        };
        if let Some(child) = child {
            log::info!("ZMODEM ssh side-channel cancellation requested; killing ssh child");
            if let Ok(mut child) = child.try_lock() {
                let _ = child.kill();
            } else {
                log::info!(
                    "ZMODEM ssh side-channel child is busy; worker will observe cancellation"
                );
            }
        }
    }

    fn is_cancelled(&self) -> bool {
        self.inner.cancelled.load(Ordering::Relaxed)
    }

    fn attach_child(&self, child: Arc<Mutex<std::process::Child>>) {
        if let Ok(mut current_child) = self.inner.child.lock() {
            *current_child = Some(child);
        }
    }

    fn detach_child(&self) {
        if let Ok(mut current_child) = self.inner.child.lock() {
            *current_child = None;
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegacySshZmodemUpload {
    pub socket_path: PathBuf,
    pub cwd: Option<String>,
    pub paths: ZmodemTransferPaths,
    pub wsl_distro: Option<String>,
}

#[derive(Clone, Debug)]
pub struct DirectSshZmodemUpload {
    pub connection_info: InteractiveSshCommand,
    pub ssh_command: Option<String>,
    pub auth: Option<DirectSshZmodemAuth>,
    pub cwd: Option<String>,
    pub detect_remote_rz_cwd: bool,
    pub remote_rz_token: Option<String>,
    pub paths: ZmodemTransferPaths,
    pub wsl_distro: Option<String>,
}

#[derive(Clone, Debug)]
pub struct DirectSshZmodemDownload {
    pub connection_info: InteractiveSshCommand,
    pub ssh_command: Option<String>,
    pub auth: Option<DirectSshZmodemAuth>,
    pub source: DirectSshZmodemDownloadSource,
    pub paths: ZmodemTransferPaths,
    pub wsl_distro: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DirectSshZmodemDownloadSource {
    Command {
        cwd: Option<String>,
        argv: Vec<String>,
    },
    DetectInteractiveSz,
}

#[derive(Clone, Debug)]
pub enum DirectSshZmodemAuth {
    Password(Zeroizing<String>),
    Passphrase(Zeroizing<String>),
}

pub enum SshZmodemUpload {
    ControlMaster(LegacySshZmodemUpload),
    Direct(DirectSshZmodemUpload),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DirectSshZmodemAuthMode {
    None,
    Password,
    Passphrase,
}

enum DirectSshAuthGuard {
    None,
    #[cfg(windows)]
    Askpass(warp_ssh_manager::AskpassSession),
}

impl DirectSshAuthGuard {
    fn kind(&self) -> &'static str {
        match self {
            Self::None => "none",
            #[cfg(windows)]
            Self::Askpass(_) => "askpass",
        }
    }
}

impl LegacySshZmodemUpload {
    pub fn new(
        socket_path: PathBuf,
        cwd: Option<String>,
        paths: ZmodemTransferPaths,
        wsl_distro: Option<String>,
    ) -> anyhow::Result<Self> {
        if paths.direction != ZmodemDirection::Upload {
            anyhow::bail!("legacy SSH side-channel only supports ZMODEM uploads");
        }
        if paths.paths.is_empty() {
            anyhow::bail!("no files selected for ZMODEM upload");
        }
        Ok(Self {
            socket_path,
            cwd,
            paths,
            wsl_distro,
        })
    }
}

impl DirectSshZmodemUpload {
    pub fn new(
        connection_info: InteractiveSshCommand,
        ssh_command: Option<String>,
        auth: Option<DirectSshZmodemAuth>,
        cwd: Option<String>,
        detect_remote_rz_cwd: bool,
        paths: ZmodemTransferPaths,
        wsl_distro: Option<String>,
    ) -> anyhow::Result<Self> {
        if paths.direction != ZmodemDirection::Upload {
            anyhow::bail!("direct SSH side-channel only supports ZMODEM uploads");
        }
        if paths.paths.is_empty() {
            anyhow::bail!("no files selected for ZMODEM upload");
        }
        if connection_info
            .host
            .as_deref()
            .filter(|host| !host.is_empty())
            .is_none()
        {
            anyhow::bail!("direct SSH side-channel requires an SSH host");
        }
        Ok(Self {
            connection_info,
            ssh_command,
            auth,
            cwd,
            detect_remote_rz_cwd,
            remote_rz_token: None,
            paths,
            wsl_distro,
        })
    }

    pub fn with_remote_rz_token(mut self, token: String) -> Self {
        self.remote_rz_token = Some(token);
        self
    }
}

pub(crate) fn remote_rz_drag_command(token: &str) -> String {
    format!(
        "\u{15}env {SSH_ZMODEM_REMOTE_RZ_TOKEN_ENV}={} rz\r",
        shell_words::quote(token)
    )
}

impl DirectSshZmodemDownload {
    pub fn new(
        connection_info: InteractiveSshCommand,
        ssh_command: Option<String>,
        auth: Option<DirectSshZmodemAuth>,
        source: DirectSshZmodemDownloadSource,
        paths: ZmodemTransferPaths,
        wsl_distro: Option<String>,
    ) -> anyhow::Result<Self> {
        if paths.direction != ZmodemDirection::Download {
            anyhow::bail!("direct SSH side-channel only supports ZMODEM downloads");
        }
        if paths.paths.is_empty() {
            anyhow::bail!("no destination directory selected for ZMODEM download");
        }
        if connection_info
            .host
            .as_deref()
            .filter(|host| !host.is_empty())
            .is_none()
        {
            anyhow::bail!("direct SSH side-channel requires an SSH host");
        }
        if let DirectSshZmodemDownloadSource::Command { argv, .. } = &source {
            if argv.is_empty() {
                anyhow::bail!("direct SSH ZMODEM download requires a remote sz command");
            }
        }
        Ok(Self {
            connection_info,
            ssh_command,
            auth,
            source,
            paths,
            wsl_distro,
        })
    }
}

pub fn run_ssh_zmodem_upload(
    upload: SshZmodemUpload,
    cancellation: LegacySshZmodemUploadCancellation,
    emit_event: impl FnMut(ZmodemEvent),
) -> anyhow::Result<()> {
    match upload {
        SshZmodemUpload::ControlMaster(upload) => {
            run_legacy_ssh_zmodem_upload(upload, cancellation, emit_event)
        }
        SshZmodemUpload::Direct(upload) => {
            run_direct_ssh_zmodem_upload(upload, cancellation, emit_event)
        }
    }
}

pub fn run_legacy_ssh_zmodem_upload(
    upload: LegacySshZmodemUpload,
    cancellation: LegacySshZmodemUploadCancellation,
    mut emit_event: impl FnMut(ZmodemEvent),
) -> anyhow::Result<()> {
    log::info!(
        "Windows legacy SSH ZMODEM upload using ControlMaster side-channel: path_count={}",
        upload.paths.paths.len()
    );

    let files = raw_upload_file_specs(&upload.paths.paths)?;
    let child = spawn_controlmaster_remote_upload_stream(&upload, &files)?;
    run_direct_ssh_raw_upload_with_child(child, files, cancellation, &mut emit_event)
}

pub fn run_direct_ssh_zmodem_upload(
    upload: DirectSshZmodemUpload,
    cancellation: LegacySshZmodemUploadCancellation,
    mut emit_event: impl FnMut(ZmodemEvent),
) -> anyhow::Result<()> {
    let host = upload
        .connection_info
        .host
        .as_deref()
        .unwrap_or("<missing-host>");
    log::info!(
        "Windows SSH ZMODEM upload using direct ssh side-channel: host={host}, path_count={}, auth_mode={:?}, has_original_ssh_command={}, wsl_distro={:?}",
        upload.paths.paths.len(),
        upload.auth_mode(),
        upload.ssh_command.is_some(),
        upload.wsl_distro
    );

    let files = raw_upload_file_specs(&upload.paths.paths)?;
    let (child, _auth_guard) = spawn_direct_remote_upload_stream(&upload, &files)?;
    run_direct_ssh_raw_upload_with_child(child, files, cancellation, &mut emit_event)
}

pub fn run_direct_ssh_zmodem_download(
    download: DirectSshZmodemDownload,
    cancellation: LegacySshZmodemUploadCancellation,
    mut emit_event: impl FnMut(ZmodemEvent),
) -> anyhow::Result<()> {
    let host = download
        .connection_info
        .host
        .as_deref()
        .unwrap_or("<missing-host>");
    log::info!(
        "Windows SSH ZMODEM download using direct ssh side-channel: host={host}, dest_count={}, auth_mode={:?}, has_original_ssh_command={}, source={:?}, wsl_distro={:?}",
        download.paths.paths.len(),
        direct_ssh_auth_mode(download.auth.as_ref()),
        download.ssh_command.is_some(),
        download.source,
        download.wsl_distro
    );

    match &download.source {
        DirectSshZmodemDownloadSource::Command { .. } => {
            let (child, _auth_guard) = spawn_direct_remote_download_stream(&download)?;
            run_direct_ssh_raw_download_with_child(
                child,
                download.paths,
                cancellation,
                &mut emit_event,
            )
        }
        DirectSshZmodemDownloadSource::DetectInteractiveSz => {
            let (child, _auth_guard) = spawn_direct_remote_sz(&download)?;
            run_zmodem_download_with_child(child, download.paths, cancellation, &mut emit_event)
        }
    }
}

fn run_zmodem_upload_with_child(
    mut child: std::process::Child,
    paths: ZmodemTransferPaths,
    cancellation: LegacySshZmodemUploadCancellation,
    mut emit_event: impl FnMut(ZmodemEvent),
) -> anyhow::Result<()> {
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| anyhow!("failed to open ssh stdin for ZMODEM upload"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("failed to open ssh stdout for ZMODEM upload"))?;
    let (stdout_rx, stdout_handle) = spawn_stdout_reader(stdout);
    let mut stderr = child.stderr.take().map(spawn_stderr_reader);
    let child = Arc::new(Mutex::new(child));
    cancellation.attach_child(child.clone());

    let mut session = ZmodemSession::upload_after_receiver_init(paths.paths)?;
    let mut saw_remote_bytes = false;
    let mut aborted_interactive_receiver = false;
    let started_at = Instant::now();
    let mut remote_bytes_read = 0u64;

    loop {
        if cancellation.is_cancelled() {
            cancel_remote_zmodem_child(&child, &mut stdin);
            cancellation.detach_child();
            emit_event(ZmodemEvent::Cancelled {
                direction: ZmodemDirection::Upload,
            });
            return Ok(());
        }

        let stdout_event = match stdout_rx.recv_timeout(SSH_ZMODEM_POLL_INTERVAL) {
            Ok(event) => Some(event),
            Err(RecvTimeoutError::Timeout) => None,
            Err(RecvTimeoutError::Disconnected) => Some(StdoutReadEvent::Eof),
        };

        let status = child
            .lock()
            .map_err(|_| anyhow!("failed to lock remote rz child"))?
            .try_wait()
            .context("failed to poll remote rz")?;
        if let Some(status) = status {
            cancellation.detach_child();
            let _ = stdout_handle.join();
            if cancellation.is_cancelled() {
                emit_event(ZmodemEvent::Cancelled {
                    direction: ZmodemDirection::Upload,
                });
                return Ok(());
            }
            if status.success() && saw_remote_bytes {
                return Ok(());
            }
            let remote_error = remote_rz_failure_message(
                "remote rz exited before ZMODEM upload completed",
                status,
                stderr.take(),
            );
            log::warn!("{remote_error}");
            anyhow::bail!(remote_error);
        }

        if !saw_remote_bytes && started_at.elapsed() >= SSH_ZMODEM_INITIAL_OUTPUT_TIMEOUT {
            kill_raw_zmodem_child(&child);
            cancellation.detach_child();
            let message = remote_rz_timeout_message(stderr.as_ref());
            let _ = stdout_handle.join();
            log::warn!("{message}");
            anyhow::bail!(message);
        }

        let bytes = match stdout_event {
            Some(StdoutReadEvent::Bytes(bytes)) => bytes,
            Some(StdoutReadEvent::Eof) => break,
            Some(StdoutReadEvent::Error(message)) if cancellation.is_cancelled() => {
                emit_event(ZmodemEvent::Cancelled {
                    direction: ZmodemDirection::Upload,
                });
                cancellation.detach_child();
                let _ = stdout_handle.join();
                return Ok(());
            }
            Some(StdoutReadEvent::Error(message)) => {
                anyhow::bail!("failed to read remote rz ZMODEM bytes: {message}");
            }
            None => continue,
        };
        saw_remote_bytes = true;
        remote_bytes_read += bytes.len() as u64;
        if !aborted_interactive_receiver {
            aborted_interactive_receiver = true;
            emit_event(ZmodemEvent::AbortInteractiveReceiver);
        }
        log::debug!(
            "ZMODEM ssh side-channel read {} byte(s) from remote rz stdout",
            bytes.len()
        );
        session.append_input(&bytes);

        let mut outbound = Vec::new();
        let completed = session.drain_actions(
            |bytes| {
                outbound.extend(bytes);
            },
            &mut emit_event,
        )?;
        if let Err(err) = write_all_zmodem_wire(&mut stdin, &outbound) {
            return Err(err).context("failed to write ZMODEM bytes to remote rz stdin");
        }

        if completed {
            drop(stdin);
            let Some(status) = wait_for_child_or_cancel(&child, &cancellation)
                .context("failed to wait for remote rz")?
            else {
                cancellation.detach_child();
                let _ = stdout_handle.join();
                emit_event(ZmodemEvent::Cancelled {
                    direction: ZmodemDirection::Upload,
                });
                return Ok(());
            };
            cancellation.detach_child();
            let _ = stdout_handle.join();
            if status.success() {
                log_zmodem_side_channel_throughput("upload", remote_bytes_read, started_at);
                return Ok(());
            }
            let remote_error = remote_rz_failure_message("remote rz failed", status, stderr.take());
            log::warn!("{remote_error}");
            anyhow::bail!(remote_error);
        }
    }

    drop(stdin);
    let Some(status) =
        wait_for_child_or_cancel(&child, &cancellation).context("failed to wait for remote rz")?
    else {
        cancellation.detach_child();
        let _ = stdout_handle.join();
        emit_event(ZmodemEvent::Cancelled {
            direction: ZmodemDirection::Upload,
        });
        return Ok(());
    };
    cancellation.detach_child();
    let _ = stdout_handle.join();
    if cancellation.is_cancelled() {
        emit_event(ZmodemEvent::Cancelled {
            direction: ZmodemDirection::Upload,
        });
        return Ok(());
    }
    if status.success() && saw_remote_bytes {
        return Ok(());
    }

    let remote_error = remote_rz_failure_message(
        "remote rz exited before ZMODEM upload completed",
        status,
        stderr.take(),
    );
    log::warn!("{remote_error}");
    anyhow::bail!(remote_error)
}

fn run_direct_ssh_raw_upload_with_child(
    mut child: std::process::Child,
    files: Vec<RawUploadFileSpec>,
    cancellation: LegacySshZmodemUploadCancellation,
    emit_event: &mut impl FnMut(ZmodemEvent),
) -> anyhow::Result<()> {
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| anyhow!("failed to open ssh stdin for raw ZMODEM upload"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("failed to open ssh stdout for raw ZMODEM upload"))?;
    let (stdout_rx, stdout_handle) = spawn_stdout_reader(stdout);
    let mut stderr = child.stderr.take().map(spawn_stderr_reader);
    let child = Arc::new(Mutex::new(child));
    cancellation.attach_child(child.clone());

    if let Err(err) = wait_for_raw_upload_ready(&child, &stdout_rx, stderr.as_ref(), &cancellation)
    {
        cancellation.detach_child();
        let _ = stdout_handle.join();
        if cancellation.is_cancelled() {
            emit_event(ZmodemEvent::Cancelled {
                direction: ZmodemDirection::Upload,
            });
            return Ok(());
        }
        return Err(err);
    }

    emit_event(ZmodemEvent::AbortInteractiveReceiver);
    emit_event(ZmodemEvent::Started {
        direction: ZmodemDirection::Upload,
    });
    let started_at = Instant::now();
    let mut uploaded_bytes = 0u64;
    let mut copy_buf = vec![0u8; SSH_ZMODEM_RAW_BUFFER_SIZE];
    let mut staged_files = Vec::new();

    for spec in files {
        if cancellation.is_cancelled() {
            kill_raw_zmodem_child(&child);
            cancellation.detach_child();
            let _ = stdout_handle.join();
            emit_event(ZmodemEvent::Cancelled {
                direction: ZmodemDirection::Upload,
            });
            return Ok(());
        }

        let mut file = File::open(&spec.path).with_context(|| {
            format!("failed to open ZMODEM upload file {}", spec.path.display())
        })?;
        emit_event(ZmodemEvent::FileStarted {
            direction: ZmodemDirection::Upload,
            name: spec.name.clone(),
            size: Some(spec.size),
            path: Some(spec.path.clone()),
        });

        let mut transferred = 0u64;
        let mut last_progress_event_at = Instant::now()
            .checked_sub(SSH_ZMODEM_PROGRESS_EVENT_INTERVAL)
            .unwrap_or_else(Instant::now);
        loop {
            if cancellation.is_cancelled() {
                kill_raw_zmodem_child(&child);
                cancellation.detach_child();
                let _ = stdout_handle.join();
                emit_event(ZmodemEvent::Cancelled {
                    direction: ZmodemDirection::Upload,
                });
                return Ok(());
            }

            let bytes_read = file.read(&mut copy_buf).with_context(|| {
                format!("failed to read ZMODEM upload file {}", spec.path.display())
            })?;
            if bytes_read == 0 {
                break;
            }
            if let Err(err) =
                write_all_raw_upload_bytes(&mut stdin, &copy_buf[..bytes_read], &cancellation)
            {
                if cancellation.is_cancelled() {
                    kill_raw_zmodem_child(&child);
                    cancellation.detach_child();
                    let _ = stdout_handle.join();
                    emit_event(ZmodemEvent::Cancelled {
                        direction: ZmodemDirection::Upload,
                    });
                    return Ok(());
                }
                let status = child
                    .lock()
                    .ok()
                    .and_then(|mut child| child.try_wait().ok().flatten());
                kill_raw_zmodem_child(&child);
                cancellation.detach_child();
                let _ = stdout_handle.join();
                let stderr_text = stderr.take().and_then(read_stderr_text);
                let message = raw_upload_write_failure_message(
                    &spec,
                    transferred,
                    &err,
                    status,
                    stderr_text.as_deref(),
                );
                log::warn!("{message}");
                anyhow::bail!(message);
            }
            transferred += bytes_read as u64;
            uploaded_bytes += bytes_read as u64;
            let now = Instant::now();
            if now.duration_since(last_progress_event_at) >= SSH_ZMODEM_PROGRESS_EVENT_INTERVAL
                || transferred >= spec.size
            {
                last_progress_event_at = now;
                emit_event(ZmodemEvent::Progress {
                    direction: ZmodemDirection::Upload,
                    name: spec.name.clone(),
                    transferred,
                    total: Some(spec.size),
                });
            }
        }
        staged_files.push((spec.name, spec.path));
    }

    if let Err(err) = wait_for_raw_upload_staged(&child, &stdout_rx, stderr.as_ref(), &cancellation)
    {
        cancellation.detach_child();
        let _ = stdout_handle.join();
        if cancellation.is_cancelled() {
            emit_event(ZmodemEvent::Cancelled {
                direction: ZmodemDirection::Upload,
            });
            return Ok(());
        }
        return Err(err);
    }

    if cancellation.is_cancelled() {
        kill_raw_zmodem_child(&child);
        cancellation.detach_child();
        let _ = stdout_handle.join();
        emit_event(ZmodemEvent::Cancelled {
            direction: ZmodemDirection::Upload,
        });
        return Ok(());
    }

    stdin
        .write_all(SSH_ZMODEM_RAW_UPLOAD_COMMIT)
        .context("failed to finish raw ZMODEM upload stream")?;
    stdin.flush().context("failed to flush raw ZMODEM upload")?;
    drop(stdin);

    let Some(status) = wait_for_child_or_cancel(&child, &cancellation)
        .context("failed to wait for raw ZMODEM upload")?
    else {
        cancellation.detach_child();
        let _ = stdout_handle.join();
        emit_event(ZmodemEvent::Cancelled {
            direction: ZmodemDirection::Upload,
        });
        return Ok(());
    };
    cancellation.detach_child();
    let _ = stdout_handle.join();

    if cancellation.is_cancelled() {
        emit_event(ZmodemEvent::Cancelled {
            direction: ZmodemDirection::Upload,
        });
        return Ok(());
    }
    if status.success() {
        log_zmodem_side_channel_throughput("raw-upload", uploaded_bytes, started_at);
        for (name, path) in staged_files {
            emit_event(ZmodemEvent::FileCompleted {
                direction: ZmodemDirection::Upload,
                name,
                path: Some(path),
            });
        }
        emit_event(ZmodemEvent::Completed {
            direction: ZmodemDirection::Upload,
        });
        return Ok(());
    }

    let remote_error = remote_rz_failure_message("remote raw upload failed", status, stderr.take());
    log::warn!("{remote_error}");
    anyhow::bail!(remote_error)
}

fn wait_for_raw_upload_ready(
    child: &Arc<Mutex<std::process::Child>>,
    stdout_rx: &mpsc::Receiver<StdoutReadEvent>,
    stderr: Option<&StderrReader>,
    cancellation: &LegacySshZmodemUploadCancellation,
) -> anyhow::Result<()> {
    wait_for_raw_upload_marker(
        child,
        stdout_rx,
        stderr,
        cancellation,
        SSH_ZMODEM_RAW_UPLOAD_READY,
        "ready",
    )
}

fn wait_for_raw_upload_staged(
    child: &Arc<Mutex<std::process::Child>>,
    stdout_rx: &mpsc::Receiver<StdoutReadEvent>,
    stderr: Option<&StderrReader>,
    cancellation: &LegacySshZmodemUploadCancellation,
) -> anyhow::Result<()> {
    wait_for_raw_upload_marker(
        child,
        stdout_rx,
        stderr,
        cancellation,
        SSH_ZMODEM_RAW_UPLOAD_STAGED,
        "staged",
    )
}

fn wait_for_raw_upload_marker(
    child: &Arc<Mutex<std::process::Child>>,
    stdout_rx: &mpsc::Receiver<StdoutReadEvent>,
    stderr: Option<&StderrReader>,
    cancellation: &LegacySshZmodemUploadCancellation,
    marker: &[u8],
    marker_name: &str,
) -> anyhow::Result<()> {
    let started_at = Instant::now();
    let mut output = Vec::new();
    loop {
        if cancellation.is_cancelled() {
            kill_raw_zmodem_child(child);
            anyhow::bail!("raw ZMODEM upload cancelled before remote side was {marker_name}");
        }
        if started_at.elapsed() >= SSH_ZMODEM_INITIAL_OUTPUT_TIMEOUT {
            kill_raw_zmodem_child(child);
            let mut message = format!(
                "remote upload helper did not become {marker_name} within {}s",
                SSH_ZMODEM_INITIAL_OUTPUT_TIMEOUT.as_secs()
            );
            if let Some(stderr_text) = stderr.and_then(StderrReader::snapshot_text) {
                message.push_str(": ");
                message.push_str(&stderr_text);
            }
            anyhow::bail!(message);
        }

        match stdout_rx.recv_timeout(SSH_ZMODEM_POLL_INTERVAL) {
            Ok(StdoutReadEvent::Bytes(bytes)) => {
                output.extend_from_slice(&bytes);
                if output.windows(marker.len()).any(|window| window == marker) {
                    return Ok(());
                }
            }
            Ok(StdoutReadEvent::Eof) => {
                anyhow::bail!("remote upload helper exited before becoming {marker_name}");
            }
            Ok(StdoutReadEvent::Error(message)) => {
                anyhow::bail!("failed to read raw upload {marker_name} marker: {message}");
            }
            Err(RecvTimeoutError::Timeout) => {
                if let Ok(mut child) = child.try_lock() {
                    let status = child
                        .try_wait()
                        .context("failed to poll raw ZMODEM upload helper")?;
                    if let Some(status) = status {
                        let message = format!(
                            "remote upload helper exited before becoming {marker_name} with exit code {}",
                            format_exit_status(status)
                        );
                        anyhow::bail!(message);
                    }
                }
            }
            Err(RecvTimeoutError::Disconnected) => {
                anyhow::bail!("remote upload helper stdout closed before becoming {marker_name}");
            }
        }
    }
}

fn write_all_raw_upload_bytes(
    stdin: &mut impl Write,
    mut bytes: &[u8],
    cancellation: &LegacySshZmodemUploadCancellation,
) -> std::io::Result<()> {
    while !bytes.is_empty() {
        if cancellation.is_cancelled() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                "raw ZMODEM upload cancelled",
            ));
        }
        let chunk_len = bytes.len().min(SSH_ZMODEM_RAW_WRITE_CHUNK_SIZE);
        stdin.write_all(&bytes[..chunk_len])?;
        bytes = &bytes[chunk_len..];
    }
    Ok(())
}

fn raw_upload_write_failure_message(
    spec: &RawUploadFileSpec,
    transferred: u64,
    error: &std::io::Error,
    status: Option<ExitStatus>,
    stderr: Option<&str>,
) -> String {
    let mut message = format!(
        "failed to write raw ZMODEM upload bytes for {} after {transferred}/{} bytes: {error}",
        spec.path.display(),
        spec.size
    );
    if let Some(status) = status {
        message.push_str("; remote helper exit code ");
        message.push_str(&format_exit_status(status));
    }
    if let Some(stderr) = stderr.filter(|stderr| !stderr.is_empty()) {
        message.push_str("; remote stderr: ");
        message.push_str(stderr);
    }
    message
}

fn run_zmodem_download_with_child(
    mut child: std::process::Child,
    paths: ZmodemTransferPaths,
    cancellation: LegacySshZmodemUploadCancellation,
    mut emit_event: impl FnMut(ZmodemEvent),
) -> anyhow::Result<()> {
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| anyhow!("failed to open ssh stdin for ZMODEM download"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("failed to open ssh stdout for ZMODEM download"))?;
    let (stdout_rx, stdout_handle) = spawn_stdout_reader(stdout);
    let mut stderr = child.stderr.take().map(spawn_stderr_reader);
    let child = Arc::new(Mutex::new(child));
    cancellation.attach_child(child.clone());

    let dest_dir = paths
        .paths
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("no destination directory selected for ZMODEM download"))?;
    let mut session = ZmodemSession::download_to_directory(dest_dir)?;
    let mut saw_remote_bytes = false;
    let mut aborted_interactive_sender = false;
    let started_at = Instant::now();
    let mut remote_bytes_read = 0u64;

    drain_zmodem_session_to_child_stdin(&mut session, &mut stdin, &mut emit_event)?;

    loop {
        if cancellation.is_cancelled() {
            cancel_remote_zmodem_child(&child, &mut stdin);
            cancellation.detach_child();
            emit_event(ZmodemEvent::Cancelled {
                direction: ZmodemDirection::Download,
            });
            return Ok(());
        }

        let stdout_event = match stdout_rx.recv_timeout(SSH_ZMODEM_POLL_INTERVAL) {
            Ok(event) => Some(event),
            Err(RecvTimeoutError::Timeout) => None,
            Err(RecvTimeoutError::Disconnected) => Some(StdoutReadEvent::Eof),
        };

        if !saw_remote_bytes && started_at.elapsed() >= SSH_ZMODEM_INITIAL_OUTPUT_TIMEOUT {
            abort_interactive_zmodem_once(&mut aborted_interactive_sender, &mut emit_event);
            kill_raw_zmodem_child(&child);
            cancellation.detach_child();
            let message = remote_sz_timeout_message(stderr.as_ref());
            let _ = stdout_handle.join();
            log::warn!("{message}");
            anyhow::bail!(message);
        }

        match stdout_event {
            Some(StdoutReadEvent::Bytes(bytes)) => {
                saw_remote_bytes = true;
                remote_bytes_read += bytes.len() as u64;
                abort_interactive_zmodem_once(&mut aborted_interactive_sender, &mut emit_event);
                log::debug!(
                    "ZMODEM ssh side-channel read {} byte(s) from remote sz stdout",
                    bytes.len()
                );
                session.append_input(&bytes);
                if drain_zmodem_session_to_child_stdin(&mut session, &mut stdin, &mut emit_event)? {
                    drop(stdin);
                    let Some(status) = wait_for_child_or_cancel(&child, &cancellation)
                        .context("failed to wait for remote sz")?
                    else {
                        cancellation.detach_child();
                        let _ = stdout_handle.join();
                        emit_event(ZmodemEvent::Cancelled {
                            direction: ZmodemDirection::Download,
                        });
                        return Ok(());
                    };
                    cancellation.detach_child();
                    let _ = stdout_handle.join();
                    if status.success() {
                        log_zmodem_side_channel_throughput(
                            "download",
                            remote_bytes_read,
                            started_at,
                        );
                        return Ok(());
                    }
                    let remote_error =
                        remote_rz_failure_message("remote sz failed", status, stderr.take());
                    log::warn!("{remote_error}");
                    anyhow::bail!(remote_error);
                }
            }
            Some(StdoutReadEvent::Eof) => {
                drop(stdin);
                let Some(status) = wait_for_child_or_cancel(&child, &cancellation)
                    .context("failed to wait for remote sz")?
                else {
                    cancellation.detach_child();
                    let _ = stdout_handle.join();
                    emit_event(ZmodemEvent::Cancelled {
                        direction: ZmodemDirection::Download,
                    });
                    return Ok(());
                };
                cancellation.detach_child();
                let _ = stdout_handle.join();
                abort_interactive_zmodem_once(&mut aborted_interactive_sender, &mut emit_event);
                if cancellation.is_cancelled() {
                    emit_event(ZmodemEvent::Cancelled {
                        direction: ZmodemDirection::Download,
                    });
                    return Ok(());
                }
                let remote_error = remote_rz_failure_message(
                    "remote sz exited before ZMODEM download completed",
                    status,
                    stderr.take(),
                );
                log::warn!("{remote_error}");
                anyhow::bail!(remote_error);
            }
            Some(StdoutReadEvent::Error(message)) if cancellation.is_cancelled() => {
                emit_event(ZmodemEvent::Cancelled {
                    direction: ZmodemDirection::Download,
                });
                cancellation.detach_child();
                let _ = stdout_handle.join();
                return Ok(());
            }
            Some(StdoutReadEvent::Error(message)) => {
                abort_interactive_zmodem_once(&mut aborted_interactive_sender, &mut emit_event);
                anyhow::bail!("failed to read remote sz ZMODEM bytes: {message}");
            }
            None => {
                let status = child
                    .lock()
                    .map_err(|_| anyhow!("failed to lock remote sz child"))?
                    .try_wait()
                    .context("failed to poll remote sz")?;
                if let Some(status) = status {
                    cancellation.detach_child();
                    abort_interactive_zmodem_once(&mut aborted_interactive_sender, &mut emit_event);
                    if cancellation.is_cancelled() {
                        emit_event(ZmodemEvent::Cancelled {
                            direction: ZmodemDirection::Download,
                        });
                        let _ = stdout_handle.join();
                        return Ok(());
                    }
                    let remote_error = remote_rz_failure_message(
                        "remote sz exited before ZMODEM download completed",
                        status,
                        stderr.take(),
                    );
                    let _ = stdout_handle.join();
                    log::warn!("{remote_error}");
                    anyhow::bail!(remote_error);
                }
            }
        }
    }
}

fn run_direct_ssh_raw_download_with_child(
    mut child: std::process::Child,
    paths: ZmodemTransferPaths,
    cancellation: LegacySshZmodemUploadCancellation,
    emit_event: &mut impl FnMut(ZmodemEvent),
) -> anyhow::Result<()> {
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("failed to open ssh stdout for raw ZMODEM download"))?;
    let (stdout_rx, stdout_handle) = spawn_stdout_reader(stdout);
    let mut stderr = child.stderr.take().map(spawn_stderr_reader);
    let child = Arc::new(Mutex::new(child));
    cancellation.attach_child(child.clone());

    let dest_dir = paths
        .paths
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("no destination directory selected for raw ZMODEM download"))?;
    let mut parser = RawDownloadParser::new(dest_dir)?;
    emit_event(ZmodemEvent::Started {
        direction: ZmodemDirection::Download,
    });
    let started_at = Instant::now();

    loop {
        if cancellation.is_cancelled() {
            kill_raw_zmodem_child(&child);
            cancellation.detach_child();
            let _ = stdout_handle.join();
            emit_event(ZmodemEvent::Cancelled {
                direction: ZmodemDirection::Download,
            });
            return Ok(());
        }

        match stdout_rx.recv_timeout(SSH_ZMODEM_POLL_INTERVAL) {
            Ok(StdoutReadEvent::Bytes(bytes)) => {
                if let Err(err) = parser.append(&bytes, emit_event) {
                    kill_raw_zmodem_child(&child);
                    cancellation.detach_child();
                    let _ = stdout_handle.join();
                    return Err(err);
                }
                if parser.completed {
                    let Some(status) = wait_for_child_or_cancel(&child, &cancellation)
                        .context("failed to wait for raw ZMODEM download")?
                    else {
                        cancellation.detach_child();
                        let _ = stdout_handle.join();
                        emit_event(ZmodemEvent::Cancelled {
                            direction: ZmodemDirection::Download,
                        });
                        return Ok(());
                    };
                    cancellation.detach_child();
                    let _ = stdout_handle.join();
                    if cancellation.is_cancelled() {
                        emit_event(ZmodemEvent::Cancelled {
                            direction: ZmodemDirection::Download,
                        });
                        return Ok(());
                    }
                    if status.success() {
                        let downloaded_bytes = parser.persist_completed_files(emit_event)?;
                        log_zmodem_side_channel_throughput(
                            "raw-download",
                            downloaded_bytes,
                            started_at,
                        );
                        emit_event(ZmodemEvent::Completed {
                            direction: ZmodemDirection::Download,
                        });
                        return Ok(());
                    }
                    let remote_error = remote_rz_failure_message(
                        "remote raw download failed",
                        status,
                        stderr.take(),
                    );
                    log::warn!("{remote_error}");
                    anyhow::bail!(remote_error);
                }
            }
            Ok(StdoutReadEvent::Eof) => {
                let Some(status) = wait_for_child_or_cancel(&child, &cancellation)
                    .context("failed to wait for raw ZMODEM download")?
                else {
                    cancellation.detach_child();
                    let _ = stdout_handle.join();
                    emit_event(ZmodemEvent::Cancelled {
                        direction: ZmodemDirection::Download,
                    });
                    return Ok(());
                };
                cancellation.detach_child();
                let _ = stdout_handle.join();
                if cancellation.is_cancelled() {
                    emit_event(ZmodemEvent::Cancelled {
                        direction: ZmodemDirection::Download,
                    });
                    return Ok(());
                }
                let remote_error = remote_rz_failure_message(
                    "remote raw download exited before completion",
                    status,
                    stderr.take(),
                );
                log::warn!("{remote_error}");
                anyhow::bail!(remote_error);
            }
            Ok(StdoutReadEvent::Error(message)) if cancellation.is_cancelled() => {
                cancellation.detach_child();
                let _ = stdout_handle.join();
                emit_event(ZmodemEvent::Cancelled {
                    direction: ZmodemDirection::Download,
                });
                return Ok(());
            }
            Ok(StdoutReadEvent::Error(message)) => {
                anyhow::bail!("failed to read raw ZMODEM download bytes: {message}");
            }
            Err(RecvTimeoutError::Timeout) => {
                let status = child
                    .lock()
                    .map_err(|_| anyhow!("failed to lock raw ZMODEM download child"))?
                    .try_wait()
                    .context("failed to poll raw ZMODEM download")?;
                if let Some(status) = status {
                    cancellation.detach_child();
                    let _ = stdout_handle.join();
                    if cancellation.is_cancelled() {
                        emit_event(ZmodemEvent::Cancelled {
                            direction: ZmodemDirection::Download,
                        });
                        return Ok(());
                    }
                    let remote_error = remote_rz_failure_message(
                        "remote raw download exited before completion",
                        status,
                        stderr.take(),
                    );
                    log::warn!("{remote_error}");
                    anyhow::bail!(remote_error);
                }
            }
            Err(RecvTimeoutError::Disconnected) => {
                anyhow::bail!("raw ZMODEM download stdout reader disconnected");
            }
        }
    }
}

enum RawDownloadState {
    Header,
    Name { size: u64, name_len: usize },
    Body { remaining: u64 },
}

struct RawDownloadParser {
    dest_dir: PathBuf,
    buffer: Vec<u8>,
    state: RawDownloadState,
    current_file: Option<RawDownloadFile>,
    completed_files: Vec<RawCompletedDownloadFile>,
    expected_total_size: u64,
    completed: bool,
}

struct RawDownloadFile {
    temp_file: tempfile::NamedTempFile,
    name: String,
    final_path: PathBuf,
    size: u64,
    transferred: u64,
    last_progress_event_at: Instant,
}

struct RawCompletedDownloadFile {
    temp_file: tempfile::NamedTempFile,
    name: String,
    final_path: PathBuf,
    size: u64,
}

impl RawDownloadParser {
    fn new(dest_dir: PathBuf) -> anyhow::Result<Self> {
        if !dest_dir.is_dir() {
            anyhow::bail!(
                "ZMODEM destination is not a directory: {}",
                dest_dir.display()
            );
        }
        Ok(Self {
            dest_dir,
            buffer: Vec::new(),
            state: RawDownloadState::Header,
            current_file: None,
            completed_files: Vec::new(),
            expected_total_size: 0,
            completed: false,
        })
    }

    fn append(
        &mut self,
        bytes: &[u8],
        emit_event: &mut impl FnMut(ZmodemEvent),
    ) -> anyhow::Result<()> {
        self.buffer.extend_from_slice(bytes);
        self.drain(emit_event)
    }

    fn drain(&mut self, emit_event: &mut impl FnMut(ZmodemEvent)) -> anyhow::Result<()> {
        loop {
            let state = std::mem::replace(&mut self.state, RawDownloadState::Header);
            match state {
                RawDownloadState::Header => {
                    let Some(newline) = self.buffer.iter().position(|byte| *byte == b'\n') else {
                        self.validate_header_bound()?;
                        self.state = RawDownloadState::Header;
                        return Ok(());
                    };
                    if newline > SSH_ZMODEM_RAW_DOWNLOAD_MAX_HEADER_LEN {
                        anyhow::bail!(
                            "raw ZMODEM download header exceeded {SSH_ZMODEM_RAW_DOWNLOAD_MAX_HEADER_LEN} bytes"
                        );
                    }
                    let line = self.buffer.drain(..=newline).collect::<Vec<_>>();
                    let line = String::from_utf8(line[..line.len() - 1].to_vec())
                        .context("raw ZMODEM download header was not valid UTF-8")?;
                    if line == "done" {
                        self.completed = true;
                        return Ok(());
                    }
                    let mut parts = line.split(' ');
                    match (parts.next(), parts.next(), parts.next(), parts.next()) {
                        (Some("file"), Some(size), Some(name_len), None) => {
                            let size = size
                                .parse::<u64>()
                                .context("raw ZMODEM download file size was invalid")?;
                            let name_len = name_len
                                .parse::<usize>()
                                .context("raw ZMODEM download file name length was invalid")?;
                            validate_raw_download_limits(size, name_len, self.expected_total_size)?;
                            self.expected_total_size =
                                self.expected_total_size.saturating_add(size);
                            self.state = RawDownloadState::Name { size, name_len };
                        }
                        _ => anyhow::bail!("raw ZMODEM download header was invalid: {line}"),
                    }
                }
                RawDownloadState::Name { size, name_len } => {
                    if self.buffer.len() < name_len {
                        self.state = RawDownloadState::Name { size, name_len };
                        return Ok(());
                    }
                    let name_bytes = self.buffer.drain(..name_len).collect::<Vec<_>>();
                    let name = crate::terminal::zmodem::sanitize_wire_file_name(&name_bytes)?;
                    let final_path = self.unused_final_path(&name);
                    let temp_file = tempfile::Builder::new()
                        .prefix(".warp-zmodem-")
                        .suffix(".part")
                        .tempfile_in(&self.dest_dir)?;
                    self.current_file = Some(RawDownloadFile {
                        temp_file,
                        name: name.clone(),
                        final_path: final_path.clone(),
                        size,
                        transferred: 0,
                        last_progress_event_at: Instant::now()
                            .checked_sub(SSH_ZMODEM_PROGRESS_EVENT_INTERVAL)
                            .unwrap_or_else(Instant::now),
                    });
                    emit_event(ZmodemEvent::FileStarted {
                        direction: ZmodemDirection::Download,
                        name,
                        size: Some(size),
                        path: Some(final_path),
                    });
                    self.state = RawDownloadState::Body { remaining: size };
                    if size == 0 {
                        self.finish_current_file()?;
                    }
                }
                RawDownloadState::Body { remaining } => {
                    if remaining == 0 {
                        self.finish_current_file()?;
                        continue;
                    }
                    if self.buffer.is_empty() {
                        self.state = RawDownloadState::Body { remaining };
                        return Ok(());
                    }
                    let bytes_to_write =
                        usize::try_from(remaining.min(self.buffer.len() as u64)).unwrap_or(0);
                    let bytes = self.buffer.drain(..bytes_to_write).collect::<Vec<_>>();
                    let progress = {
                        let file = self
                            .current_file
                            .as_mut()
                            .ok_or_else(|| anyhow!("raw ZMODEM download file is not open"))?;
                        file.temp_file.write_all(&bytes)?;
                        file.transferred += bytes.len() as u64;
                        let now = Instant::now();
                        if now.duration_since(file.last_progress_event_at)
                            >= SSH_ZMODEM_PROGRESS_EVENT_INTERVAL
                            || file.transferred >= file.size
                        {
                            file.last_progress_event_at = now;
                            Some((file.name.clone(), file.transferred, file.size))
                        } else {
                            None
                        }
                    };
                    if let Some((name, transferred, total)) = progress {
                        emit_event(ZmodemEvent::Progress {
                            direction: ZmodemDirection::Download,
                            name,
                            transferred,
                            total: Some(total),
                        });
                    }
                    self.state = RawDownloadState::Body {
                        remaining: remaining - bytes.len() as u64,
                    };
                }
            }
        }
    }

    fn finish_current_file(&mut self) -> anyhow::Result<()> {
        if let Some(mut file) = self.current_file.take() {
            file.temp_file.flush()?;
            self.completed_files.push(RawCompletedDownloadFile {
                temp_file: file.temp_file,
                name: file.name,
                final_path: file.final_path,
                size: file.size,
            });
        }
        self.state = RawDownloadState::Header;
        Ok(())
    }

    fn validate_header_bound(&self) -> anyhow::Result<()> {
        if self.buffer.len() > SSH_ZMODEM_RAW_DOWNLOAD_MAX_HEADER_LEN {
            anyhow::bail!(
                "raw ZMODEM download header exceeded {SSH_ZMODEM_RAW_DOWNLOAD_MAX_HEADER_LEN} bytes"
            );
        }
        Ok(())
    }

    fn unused_final_path(&self, name: &str) -> PathBuf {
        let reserved_paths = self
            .completed_files
            .iter()
            .map(|file| file.final_path.clone())
            .chain(self.current_file.iter().map(|file| file.final_path.clone()))
            .collect::<Vec<_>>();
        unused_path_avoiding_reserved(&self.dest_dir, name, &reserved_paths)
    }

    fn persist_completed_files(
        &mut self,
        emit_event: &mut impl FnMut(ZmodemEvent),
    ) -> anyhow::Result<u64> {
        let mut total = 0u64;
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
            total += file.size;
            log::debug!(
                "ZMODEM raw download committed completed file {} to {}",
                file.name,
                final_path.display()
            );
            emit_event(ZmodemEvent::FileCompleted {
                direction: ZmodemDirection::Download,
                name: file.name,
                path: Some(final_path),
            });
        }
        Ok(total)
    }
}

fn validate_raw_download_limits(
    size: u64,
    name_len: usize,
    current_total_size: u64,
) -> anyhow::Result<()> {
    if name_len == 0 {
        anyhow::bail!("raw ZMODEM download file name was empty");
    }
    if name_len > SSH_ZMODEM_RAW_DOWNLOAD_MAX_NAME_LEN {
        anyhow::bail!(
            "raw ZMODEM download file name exceeded {SSH_ZMODEM_RAW_DOWNLOAD_MAX_NAME_LEN} bytes"
        );
    }
    if size > SSH_ZMODEM_RAW_DOWNLOAD_MAX_FILE_SIZE {
        anyhow::bail!(
            "raw ZMODEM download file exceeded {SSH_ZMODEM_RAW_DOWNLOAD_MAX_FILE_SIZE} bytes"
        );
    }
    let next_total_size = current_total_size
        .checked_add(size)
        .ok_or_else(|| anyhow!("raw ZMODEM download total size overflowed"))?;
    if next_total_size > SSH_ZMODEM_RAW_DOWNLOAD_MAX_TOTAL_SIZE {
        anyhow::bail!(
            "raw ZMODEM download total size exceeded {SSH_ZMODEM_RAW_DOWNLOAD_MAX_TOTAL_SIZE} bytes"
        );
    }
    Ok(())
}

fn unused_path_avoiding_reserved(
    dest_dir: &Path,
    file_name: &str,
    reserved_paths: &[PathBuf],
) -> PathBuf {
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

fn raw_upload_file_specs(paths: &[PathBuf]) -> anyhow::Result<Vec<RawUploadFileSpec>> {
    let mut specs = Vec::with_capacity(paths.len());
    let mut seen_names: HashMap<String, PathBuf> = HashMap::with_capacity(paths.len());

    for path in paths {
        let metadata = path.metadata()?;
        if !metadata.is_file() {
            anyhow::bail!("ZMODEM upload only supports files: {}", path.display());
        }

        let name = local_upload_file_name(path)?;
        if let Some(first_path) = seen_names.get(&name) {
            let first_path = first_path.display();
            let second_path = path.display();
            anyhow::bail!(
                "ZMODEM upload contains multiple files named {name}; rename one before uploading: {first_path} and {second_path}"
            );
        }

        seen_names.insert(name.clone(), path.clone());
        specs.push(RawUploadFileSpec {
            path: path.clone(),
            name,
            size: metadata.len(),
        });
    }

    Ok(specs)
}

fn local_upload_file_name(path: &Path) -> anyhow::Result<String> {
    let file_name = path
        .file_name()
        .ok_or_else(|| anyhow!("file has no name: {}", path.display()))?
        .to_string_lossy()
        .to_string();
    crate::terminal::zmodem::sanitize_wire_file_name(file_name.as_bytes())
}

fn wait_for_child_or_cancel(
    child: &Arc<Mutex<std::process::Child>>,
    cancellation: &LegacySshZmodemUploadCancellation,
) -> anyhow::Result<Option<ExitStatus>> {
    loop {
        if cancellation.is_cancelled() {
            kill_raw_zmodem_child(child);
            return Ok(None);
        }
        let status = child
            .lock()
            .map_err(|_| anyhow!("failed to lock ZMODEM side-channel child"))?
            .try_wait()
            .context("failed to poll ZMODEM side-channel child")?;
        if let Some(status) = status {
            return Ok(Some(status));
        }
        std::thread::sleep(SSH_ZMODEM_POLL_INTERVAL);
    }
}

fn kill_raw_zmodem_child(child: &Arc<Mutex<std::process::Child>>) {
    if let Ok(mut child) = child.lock() {
        let _ = child.kill();
        let _ = child.wait();
    }
}

fn remote_raw_upload_command(
    cwd: Option<&str>,
    detect_remote_rz_cwd: bool,
    remote_rz_token: Option<&str>,
    files: &[RawUploadFileSpec],
) -> String {
    let target = if let Some(cwd) = cwd.filter(|cwd| !cwd.is_empty()) {
        format!("target={}", shell_words::quote(cwd))
    } else if let Some(token) = remote_rz_token {
        format!("target=\"$({})\"", remote_rz_token_cwd_script(token))
    } else if detect_remote_rz_cwd {
        format!("target=\"$({REMOTE_RZ_CWD_SCRIPT})\"")
    } else {
        String::from("target=$(pwd)")
    };
    let mut command = format!("{target} && {REMOTE_RAW_UPLOAD_STAGED_PREFIX}");
    for file in files {
        command.push_str("receive_file ");
        command.push_str(&file.size.to_string());
        command.push(' ');
        command.push_str(&shell_words::quote(&file.name));
        command.push_str(" || exit 1; ");
    }
    command.push_str(REMOTE_RAW_UPLOAD_STAGED_SUFFIX);
    command
}

fn remote_raw_download_command(source: &DirectSshZmodemDownloadSource) -> String {
    match source {
        DirectSshZmodemDownloadSource::Command { cwd, argv } => {
            let files = argv
                .iter()
                .skip(1)
                .map(|arg| shell_words::quote(arg).into_owned())
                .collect::<Vec<_>>()
                .join(" ");
            let script = format!(
                r#"set -- {files}; for f do [ -f "$f" ] || {{ printf '%s\n' "not a regular file: $f" >&2; exit 1; }}; size=$(wc -c < "$f") || exit 1; base=${{f##*/}}; name_len=$(printf '%s' "$base" | wc -c) || exit 1; printf 'file %s %s\n' "$size" "$name_len" || exit 1; printf '%s' "$base" || exit 1; cat -- "$f" || exit 1; done; printf 'done\n'"#
            );
            if let Some(cwd) = cwd.as_deref().filter(|cwd| !cwd.is_empty()) {
                format!("cd -- {} && {script}", shell_words::quote(cwd))
            } else {
                script
            }
        }
        DirectSshZmodemDownloadSource::DetectInteractiveSz => REMOTE_SZ_DETECT_SCRIPT.to_string(),
    }
}

const REMOTE_RAW_UPLOAD_STAGED_PREFIX: &str = concat!(
    r#"cd -- "$target" && target=$(pwd) && "#,
    r#"printf 'WARP_ZMODEM_TARGET=%s\n' "$target" >&2 && "#,
    r#"tmp=$(mktemp -d ".warp-zmodem.XXXXXX") && tmp=$(cd -- "$tmp" && pwd) || exit 1; "#,
    r#"cleanup() { rm -rf -- "$tmp"; }; trap cleanup EXIT HUP INT TERM; "#,
    r#"receive_file() { size=$1; name=$2; "#,
    r#"case "$name" in ""|.|..|*/*) exit 2;; esac; "#,
    r#"out="$tmp/$name"; "#,
    r#"dd bs=1048576 count="$size" iflag=count_bytes,fullblock of="$out" status=none || exit 1; "#,
    r#"[ "$(wc -c < "$out")" = "$size" ] || exit 1; "#,
    r#"}; "#,
    r#"printf 'WARP_ZMODEM_READY\n' && "#,
);

const REMOTE_RAW_UPLOAD_STAGED_SUFFIX: &str = concat!(
    r#"printf 'WARP_ZMODEM_STAGED\n' || exit 1; "#,
    r#"IFS= read -r marker || exit 1; "#,
    r#"[ "$marker" = "WARP_ZMODEM_COMMIT" ] || exit 1; "#,
    r#"for f in "$tmp"/* "$tmp"/.[!.]* "$tmp"/..?*; do [ -e "$f" ] || continue; base=${f##*/}; [ ! -e "$target/$base" ] || { printf '%s\n' "ZMODEM upload target already exists: $base" >&2; exit 3; }; done; "#,
    r#"for f in "$tmp"/* "$tmp"/.[!.]* "$tmp"/..?*; do [ -e "$f" ] || continue; base=${f##*/}; ln -- "$f" "$target/$base" || exit 1; rm -f -- "$f" || exit 1; done; "#,
    r#"trap - EXIT HUP INT TERM; cleanup"#,
);

fn drain_zmodem_session_to_child_stdin(
    session: &mut ZmodemSession,
    stdin: &mut impl Write,
    mut emit_event: impl FnMut(ZmodemEvent),
) -> anyhow::Result<bool> {
    let mut outbound = Vec::new();
    let completed = session.drain_actions(
        |bytes| {
            outbound.extend(bytes);
        },
        &mut emit_event,
    )?;
    if let Err(err) = write_all_zmodem_wire(stdin, &outbound) {
        return Err(err).context("failed to write ZMODEM bytes to remote process stdin");
    }
    Ok(completed)
}

fn write_all_zmodem_wire(stdin: &mut impl Write, bytes: &[u8]) -> std::io::Result<()> {
    if bytes.is_empty() {
        return Ok(());
    }
    stdin.write_all(bytes)?;
    stdin.flush()
}

fn cancel_remote_zmodem_child(child: &Arc<Mutex<std::process::Child>>, stdin: &mut impl Write) {
    let _ = stdin.write_all(ZMODEM_ABORT_SEQUENCE);
    let _ = stdin.flush();
    if let Ok(mut child) = child.lock() {
        let _ = child.kill();
        let _ = child.wait();
    }
}

fn abort_interactive_zmodem_once(aborted: &mut bool, emit_event: &mut impl FnMut(ZmodemEvent)) {
    if !*aborted {
        *aborted = true;
        emit_event(ZmodemEvent::AbortInteractiveReceiver);
    }
}

fn log_zmodem_side_channel_throughput(direction: &str, bytes: u64, started_at: Instant) {
    let elapsed = started_at.elapsed();
    let elapsed_secs = elapsed.as_secs_f64();
    let bytes_per_second = if elapsed_secs > 0.0 {
        bytes as f64 / elapsed_secs
    } else {
        0.0
    };
    log::info!(
        "ZMODEM ssh side-channel {direction} completed: wire_bytes={bytes}, elapsed_ms={}, throughput_bytes_per_second={bytes_per_second:.0}",
        elapsed.as_millis()
    );
}

fn spawn_stdout_reader(
    mut stdout: ChildStdout,
) -> (mpsc::Receiver<StdoutReadEvent>, JoinHandle<()>) {
    let (tx, rx) = mpsc::channel();
    let handle = std::thread::spawn(move || {
        let mut output_buf = vec![0u8; SSH_ZMODEM_RAW_BUFFER_SIZE];
        loop {
            match stdout.read(&mut output_buf) {
                Ok(0) => {
                    let _ = tx.send(StdoutReadEvent::Eof);
                    break;
                }
                Ok(bytes_read) => {
                    if tx
                        .send(StdoutReadEvent::Bytes(output_buf[..bytes_read].to_vec()))
                        .is_err()
                    {
                        break;
                    }
                }
                Err(err) => {
                    let _ = tx.send(StdoutReadEvent::Error(err.to_string()));
                    break;
                }
            }
        }
    });
    (rx, handle)
}

fn spawn_stderr_reader(mut stderr: ChildStderr) -> StderrReader {
    let buffer = Arc::new(Mutex::new(Vec::new()));
    let buffer_for_thread = buffer.clone();
    let handle = std::thread::spawn(move || {
        let mut stderr_bytes = Vec::new();
        let mut read_buf = [0u8; 1024];
        loop {
            match stderr.read(&mut read_buf) {
                Ok(0) => break,
                Ok(bytes_read) => {
                    stderr_bytes.extend_from_slice(&read_buf[..bytes_read]);
                    if stderr_bytes.len() > SSH_ZMODEM_STDERR_LIMIT {
                        let overflow = stderr_bytes.len() - SSH_ZMODEM_STDERR_LIMIT;
                        stderr_bytes.drain(..overflow);
                    }
                    if let Ok(mut buffer) = buffer_for_thread.lock() {
                        *buffer = stderr_bytes.clone();
                    }
                    if let Some(stderr_text) = sanitize_stderr(&stderr_bytes) {
                        log::info!("ZMODEM ssh side-channel stderr: {stderr_text}");
                    }
                }
                Err(_) => break,
            }
        }
        stderr_bytes
    });
    StderrReader { buffer, handle }
}

fn remote_rz_failure_message(
    context: &str,
    status: ExitStatus,
    stderr: Option<StderrReader>,
) -> String {
    let mut message = format!("{context} with exit code {}", format_exit_status(status));
    if let Some(stderr_text) = stderr.and_then(read_stderr_text) {
        message.push_str(": ");
        message.push_str(&stderr_text);
    }
    message
}

fn remote_rz_timeout_message(stderr: Option<&StderrReader>) -> String {
    let mut message = format!(
        "remote rz did not send ZMODEM initialization within {}s",
        SSH_ZMODEM_INITIAL_OUTPUT_TIMEOUT.as_secs()
    );
    if let Some(stderr_text) = stderr.and_then(StderrReader::snapshot_text) {
        message.push_str(": ");
        message.push_str(&stderr_text);
    }
    message
}

fn remote_sz_timeout_message(stderr: Option<&StderrReader>) -> String {
    let mut message = format!(
        "remote sz did not send ZMODEM data within {}s",
        SSH_ZMODEM_INITIAL_OUTPUT_TIMEOUT.as_secs()
    );
    if let Some(stderr_text) = stderr.and_then(StderrReader::snapshot_text) {
        message.push_str(": ");
        message.push_str(&stderr_text);
    }
    message
}

fn read_stderr_text(stderr: StderrReader) -> Option<String> {
    let bytes = stderr.handle.join().ok()?;
    sanitize_stderr(&bytes)
}

fn sanitize_stderr(bytes: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(bytes);
    let text = text.trim_matches(|ch: char| ch.is_whitespace() || ch == '\0');
    if text.is_empty() {
        return None;
    }
    let mut text = text.replace('\r', "");
    if text.len() > SSH_ZMODEM_STDERR_LIMIT {
        text.truncate(SSH_ZMODEM_STDERR_LIMIT);
        text.push_str("...");
    }
    Some(text)
}

fn format_exit_status(status: ExitStatus) -> String {
    match status.code() {
        Some(code) => code.to_string(),
        None => String::from("unknown"),
    }
}

fn spawn_controlmaster_remote_rz(
    upload: &LegacySshZmodemUpload,
) -> anyhow::Result<std::process::Child> {
    let mut command = ssh_command(upload.wsl_distro.as_deref());
    command
        .arg("-T")
        .args(remote_server::ssh::ssh_args(&upload.socket_path))
        .arg(remote_rz_command(upload.cwd.as_deref(), false))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(windows)]
    command.kill_on_parent_process_close();

    command
        .spawn()
        .context("failed to spawn ssh side-channel for ZMODEM upload")
}

fn spawn_controlmaster_remote_upload_stream(
    upload: &LegacySshZmodemUpload,
    files: &[RawUploadFileSpec],
) -> anyhow::Result<std::process::Child> {
    let remote_command = remote_raw_upload_command(upload.cwd.as_deref(), false, None, files);
    log::info!(
        "ZMODEM ControlMaster side-channel spawning raw upload helper: wsl_distro={:?}",
        upload.wsl_distro
    );
    let mut command = ssh_command(upload.wsl_distro.as_deref());
    command
        .arg("-T")
        .args(remote_server::ssh::ssh_args(&upload.socket_path))
        .arg(remote_command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(windows)]
    command.kill_on_parent_process_close();

    command
        .spawn()
        .context("failed to spawn ssh side-channel for raw ZMODEM upload")
}

fn spawn_direct_remote_rz(
    upload: &DirectSshZmodemUpload,
) -> anyhow::Result<(std::process::Child, DirectSshAuthGuard)> {
    let args = direct_ssh_args(upload)?;
    log::info!(
        "ZMODEM direct ssh side-channel spawning remote rz: program={}, args={args:?}, auth_mode={:?}, wsl_distro={:?}",
        if upload.wsl_distro.is_some() {
            "wsl"
        } else {
            "ssh"
        },
        upload.auth_mode(),
        upload.wsl_distro
    );
    let mut command = ssh_command(upload.wsl_distro.as_deref());
    command
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let auth_guard = configure_direct_ssh_auth(&mut command, upload)?;
    log::info!(
        "ZMODEM direct ssh side-channel auth environment configured: guard={}",
        auth_guard.kind()
    );

    #[cfg(windows)]
    command.kill_on_parent_process_close();

    let child = command
        .spawn()
        .context("failed to spawn direct ssh side-channel for ZMODEM upload")?;
    Ok((child, auth_guard))
}

fn spawn_direct_remote_upload_stream(
    upload: &DirectSshZmodemUpload,
    files: &[RawUploadFileSpec],
) -> anyhow::Result<(std::process::Child, DirectSshAuthGuard)> {
    let remote_command = remote_raw_upload_command(
        upload.cwd.as_deref(),
        upload.detect_remote_rz_cwd,
        upload.remote_rz_token.as_deref(),
        files,
    );
    let args = direct_ssh_args_for_remote_command(
        &upload.connection_info,
        upload.ssh_command.as_deref(),
        &remote_command,
        upload.auth_mode(),
    )?;
    log::info!(
        "ZMODEM direct ssh side-channel spawning raw upload helper: program={}, args={args:?}, auth_mode={:?}, wsl_distro={:?}",
        if upload.wsl_distro.is_some() {
            "wsl"
        } else {
            "ssh"
        },
        upload.auth_mode(),
        upload.wsl_distro
    );
    let mut command = ssh_command(upload.wsl_distro.as_deref());
    command
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let auth_guard = configure_direct_ssh_auth(&mut command, upload)?;
    log::info!(
        "ZMODEM direct ssh side-channel auth environment configured: guard={}",
        auth_guard.kind()
    );

    #[cfg(windows)]
    command.kill_on_parent_process_close();

    let child = command
        .spawn()
        .context("failed to spawn direct ssh side-channel for raw ZMODEM upload")?;
    Ok((child, auth_guard))
}

fn spawn_direct_remote_sz(
    download: &DirectSshZmodemDownload,
) -> anyhow::Result<(std::process::Child, DirectSshAuthGuard)> {
    let args = direct_ssh_download_args(download)?;
    log::info!(
        "ZMODEM direct ssh side-channel spawning remote sz: program={}, args={args:?}, auth_mode={:?}, wsl_distro={:?}",
        if download.wsl_distro.is_some() {
            "wsl"
        } else {
            "ssh"
        },
        direct_ssh_auth_mode(download.auth.as_ref()),
        download.wsl_distro
    );
    let mut command = ssh_command(download.wsl_distro.as_deref());
    command
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let auth_guard = configure_direct_ssh_auth_for(
        &mut command,
        download.auth.as_ref(),
        direct_ssh_auth_mode(download.auth.as_ref()),
        download.wsl_distro.as_deref(),
    )?;
    log::info!(
        "ZMODEM direct ssh side-channel auth environment configured: guard={}",
        auth_guard.kind()
    );

    #[cfg(windows)]
    command.kill_on_parent_process_close();

    let child = command
        .spawn()
        .context("failed to spawn direct ssh side-channel for ZMODEM download")?;
    Ok((child, auth_guard))
}

fn spawn_direct_remote_download_stream(
    download: &DirectSshZmodemDownload,
) -> anyhow::Result<(std::process::Child, DirectSshAuthGuard)> {
    let remote_command = remote_raw_download_command(&download.source);
    let args = direct_ssh_args_for_remote_command(
        &download.connection_info,
        download.ssh_command.as_deref(),
        &remote_command,
        direct_ssh_auth_mode(download.auth.as_ref()),
    )?;
    log::info!(
        "ZMODEM direct ssh side-channel spawning raw download helper: program={}, args={args:?}, auth_mode={:?}, wsl_distro={:?}",
        if download.wsl_distro.is_some() {
            "wsl"
        } else {
            "ssh"
        },
        direct_ssh_auth_mode(download.auth.as_ref()),
        download.wsl_distro
    );
    let mut command = ssh_command(download.wsl_distro.as_deref());
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let auth_guard = configure_direct_ssh_auth_for(
        &mut command,
        download.auth.as_ref(),
        direct_ssh_auth_mode(download.auth.as_ref()),
        download.wsl_distro.as_deref(),
    )?;
    log::info!(
        "ZMODEM direct ssh side-channel auth environment configured: guard={}",
        auth_guard.kind()
    );

    #[cfg(windows)]
    command.kill_on_parent_process_close();

    let child = command
        .spawn()
        .context("failed to spawn direct ssh side-channel for raw ZMODEM download")?;
    Ok((child, auth_guard))
}

fn configure_direct_ssh_auth(
    command: &mut command::blocking::Command,
    upload: &DirectSshZmodemUpload,
) -> anyhow::Result<DirectSshAuthGuard> {
    configure_direct_ssh_auth_for(
        command,
        upload.auth.as_ref(),
        upload.auth_mode(),
        upload.wsl_distro.as_deref(),
    )
}

fn configure_direct_ssh_auth_for(
    command: &mut command::blocking::Command,
    auth: Option<&DirectSshZmodemAuth>,
    auth_mode: DirectSshZmodemAuthMode,
    wsl_distro: Option<&str>,
) -> anyhow::Result<DirectSshAuthGuard> {
    #[cfg(windows)]
    {
        let Some(auth) = auth else {
            log::info!("ZMODEM direct ssh auth configuration skipped: auth_mode=None");
            return Ok(DirectSshAuthGuard::None);
        };
        if wsl_distro.is_some() {
            log::warn!(
                "ZMODEM direct ssh auth configuration cannot reuse password for WSL-backed session"
            );
            anyhow::bail!(
                "direct SSH side-channel password reuse is not supported for WSL-backed sessions"
            );
        }
        let secret = match auth {
            DirectSshZmodemAuth::Password(secret) | DirectSshZmodemAuth::Passphrase(secret) => {
                secret
            }
        };
        log::info!(
            "ZMODEM direct ssh preparing SSH_ASKPASS: auth_mode={:?}, secret_len={}",
            auth_mode,
            secret.len()
        );
        let askpass = warp_ssh_manager::AskpassSession::new(secret)
            .context("failed to prepare SSH_ASKPASS for direct ssh side-channel")?;
        askpass.apply_env_blocking(command);
        log::info!("ZMODEM direct ssh SSH_ASKPASS environment applied");
        return Ok(DirectSshAuthGuard::Askpass(askpass));
    }

    #[cfg(not(windows))]
    {
        let _ = command;
        let _ = auth;
        let _ = auth_mode;
        let _ = wsl_distro;
        Ok(DirectSshAuthGuard::None)
    }
}

fn ssh_command(wsl_distro: Option<&str>) -> command::blocking::Command {
    if let Some(wsl_distro) = wsl_distro {
        let mut command = command::blocking::Command::new("wsl");
        command.args(["-d", wsl_distro, "-e", "ssh"]);
        command
    } else {
        command::blocking::Command::new("ssh")
    }
}

fn direct_ssh_args(upload: &DirectSshZmodemUpload) -> anyhow::Result<Vec<String>> {
    let remote_command = remote_rz_command(upload.cwd.as_deref(), upload.detect_remote_rz_cwd);
    direct_ssh_args_for_remote_command(
        &upload.connection_info,
        upload.ssh_command.as_deref(),
        &remote_command,
        upload.auth_mode(),
    )
}

fn direct_ssh_download_args(download: &DirectSshZmodemDownload) -> anyhow::Result<Vec<String>> {
    let remote_command = remote_sz_command(&download.source);
    direct_ssh_args_for_remote_command(
        &download.connection_info,
        download.ssh_command.as_deref(),
        &remote_command,
        direct_ssh_auth_mode(download.auth.as_ref()),
    )
}

fn direct_ssh_args_for_remote_command(
    connection_info: &InteractiveSshCommand,
    ssh_command: Option<&str>,
    remote_command: &str,
    auth_mode: DirectSshZmodemAuthMode,
) -> anyhow::Result<Vec<String>> {
    if let Some(args) = ssh_command.and_then(|command| {
        direct_ssh_args_from_interactive_command_with_remote_command(
            command,
            remote_command,
            auth_mode,
        )
    }) {
        return Ok(args);
    }

    let host = connection_info
        .host
        .as_deref()
        .filter(|host| !host.is_empty())
        .ok_or_else(|| anyhow!("direct SSH side-channel requires an SSH host"))?;
    let mut args = Vec::new();
    append_direct_ssh_side_channel_options(&mut args, auth_mode);
    if let Some(port) = connection_info
        .port
        .as_deref()
        .filter(|port| !port.is_empty())
    {
        args.push("-p".to_string());
        args.push(port.to_string());
    }
    args.push(host.to_string());
    args.push(remote_command.to_string());
    Ok(args)
}

impl DirectSshZmodemUpload {
    fn auth_mode(&self) -> DirectSshZmodemAuthMode {
        direct_ssh_auth_mode(self.auth.as_ref())
    }
}

fn direct_ssh_auth_mode(auth: Option<&DirectSshZmodemAuth>) -> DirectSshZmodemAuthMode {
    match auth {
        Some(DirectSshZmodemAuth::Password(_)) => DirectSshZmodemAuthMode::Password,
        Some(DirectSshZmodemAuth::Passphrase(_)) => DirectSshZmodemAuthMode::Passphrase,
        None => DirectSshZmodemAuthMode::None,
    }
}

fn direct_ssh_args_from_interactive_command(
    command: &str,
    cwd: Option<&str>,
    detect_remote_rz_cwd: bool,
    auth_mode: DirectSshZmodemAuthMode,
) -> Option<Vec<String>> {
    let remote_command = remote_rz_command(cwd, detect_remote_rz_cwd);
    direct_ssh_args_from_interactive_command_with_remote_command(
        command,
        &remote_command,
        auth_mode,
    )
}

fn direct_ssh_args_from_interactive_command_with_remote_command(
    command: &str,
    remote_command: &str,
    auth_mode: DirectSshZmodemAuthMode,
) -> Option<Vec<String>> {
    let mut tokens = shell_words::split(command).ok()?;
    if tokens.first().is_some_and(|token| token == "command") {
        tokens.remove(0);
    }
    if tokens.first().is_none_or(|token| token != "ssh") {
        return None;
    }

    let mut args = Vec::new();
    let mut host = None;
    let mut i = 1;
    while i < tokens.len() {
        match tokens[i].as_str() {
            "-T" | "-W" => return None,
            "-t" | "-tt" => {}
            "-p" | "-B" | "-b" | "-c" | "-D" | "-E" | "-e" | "-F" | "-I" | "-i" | "-J" | "-L"
            | "-l" | "-m" | "-O" | "-o" | "-P" | "-Q" | "-R" | "-S" | "-w" => {
                args.push(tokens[i].clone());
                i += 1;
                if i >= tokens.len() {
                    return None;
                }
                args.push(tokens[i].clone());
            }
            arg if arg.starts_with('-') => {
                args.push(arg.to_string());
            }
            positional => {
                if host.is_some() {
                    return None;
                }
                host = Some(positional.to_string());
            }
        }
        i += 1;
    }

    append_direct_ssh_side_channel_options(&mut args, auth_mode);
    args.push(host?);
    args.push(remote_command.to_string());
    Some(args)
}

fn append_direct_ssh_side_channel_options(
    args: &mut Vec<String>,
    auth_mode: DirectSshZmodemAuthMode,
) {
    args.push("-T".to_string());
    args.push("-o".to_string());
    args.push(match auth_mode {
        DirectSshZmodemAuthMode::None => "BatchMode=yes".to_string(),
        DirectSshZmodemAuthMode::Password | DirectSshZmodemAuthMode::Passphrase => {
            "BatchMode=no".to_string()
        }
    });
    if matches!(auth_mode, DirectSshZmodemAuthMode::Password) {
        args.push("-o".to_string());
        args.push("PreferredAuthentications=password".to_string());
        args.push("-o".to_string());
        args.push("KbdInteractiveAuthentication=no".to_string());
        args.push("-o".to_string());
        args.push("NumberOfPasswordPrompts=1".to_string());
    }
    args.push("-o".to_string());
    args.push("ForwardX11=no".to_string());
}

fn remote_rz_command(cwd: Option<&str>, detect_remote_rz_cwd: bool) -> String {
    let target = if let Some(cwd) = cwd.filter(|cwd| !cwd.is_empty()) {
        format!("target={}", shell_words::quote(cwd))
    } else if detect_remote_rz_cwd {
        format!("target=\"$({REMOTE_RZ_CWD_SCRIPT})\"")
    } else {
        String::from("target=$(pwd)")
    };
    format!("{target} && {REMOTE_RZ_STAGED_UPLOAD_SCRIPT}")
}

const REMOTE_RZ_STAGED_UPLOAD_SCRIPT: &str = concat!(
    r#"cd -- "$target" && target=$(pwd) && "#,
    r#"tmp=$(mktemp -d ".warp-zmodem.XXXXXX") && tmp=$(cd -- "$tmp" && pwd) || exit 1; "#,
    r#"cleanup() { rm -rf -- "$tmp"; }; trap cleanup EXIT HUP INT TERM; "#,
    r#"cd -- "$tmp" && "#,
    "rz -q -y",
    r#"; status=$?; if [ "$status" -ne 0 ]; then exit "$status"; fi; "#,
    r#"for f in ./* ./.[!.]* ./..?*; do [ -e "$f" ] || continue; base=${f##*/}; [ ! -e "$target/$base" ] || { printf '%s\n' "ZMODEM upload target already exists: $base" >&2; exit 3; }; done; "#,
    r#"for f in ./* ./.[!.]* ./..?*; do [ -e "$f" ] || continue; base=${f##*/}; ln -- "$f" "$target/$base" || exit 1; rm -f -- "$f" || exit 1; done; "#,
    r#"trap - EXIT HUP INT TERM; cleanup"#,
);

const REMOTE_RZ_CWD_SCRIPT: &str = concat!(
    r#"rz_cwd=; for p in /proc/[0-9]*; do "#,
    r#"comm=$(cat "$p/comm" 2>/dev/null) || continue; "#,
    r#"exe=$(basename "$(readlink "$p/exe" 2>/dev/null)" 2>/dev/null); "#,
    r#"matched=; case "$comm" in rz|lrz) matched=1;; esac; "#,
    r#"case "$exe" in rz|lrz) matched=1;; esac; "#,
    r#"if [ -z "$matched" ]; then cmd0=$(tr '\000' '\n' < "$p/cmdline" 2>/dev/null | sed -n '1p'); cmd0=$(basename "$cmd0" 2>/dev/null); case "$cmd0" in rz|lrz) matched=1;; esac; fi; "#,
    r#"[ -n "$matched" ] || continue; cwd=$(readlink "$p/cwd" 2>/dev/null) || continue; [ -n "$cwd" ] || continue; "#,
    r#"if [ -n "$rz_cwd" ] && [ "$rz_cwd" != "$cwd" ]; then printf '%s\n' 'multiple active remote rz working directories; refusing to choose another SSH session' >&2; exit 1; fi; "#,
    r#"rz_cwd=$cwd; done; "#,
    r#"[ -n "$rz_cwd" ] || { printf '%s\n' 'no active remote rz process found' >&2; exit 1; }; "#,
    r#"printf '%s\n' "$rz_cwd""#,
);

fn remote_rz_token_cwd_script(token: &str) -> String {
    format!(
        r#"expected_token={}; attempt=0; while [ "$attempt" -lt 100 ]; do for p in /proc/[0-9]*; do [ -r "$p/environ" ] || continue; tr '\000' '\n' < "$p/environ" 2>/dev/null | grep -Fqx "{}=$expected_token" || continue; cwd=$(readlink "$p/cwd" 2>/dev/null) || continue; [ -n "$cwd" ] || continue; printf '%s\n' "$cwd"; exit 0; done; attempt=$((attempt + 1)); sleep 0.1; done; printf '%s\n' 'no matching remote rz process found for drag-and-drop upload' >&2; exit 1"#,
        shell_words::quote(token),
        SSH_ZMODEM_REMOTE_RZ_TOKEN_ENV,
    )
}

fn remote_sz_command(source: &DirectSshZmodemDownloadSource) -> String {
    match source {
        DirectSshZmodemDownloadSource::Command { cwd, argv } => {
            let command = argv
                .iter()
                .map(|arg| shell_words::quote(arg).into_owned())
                .collect::<Vec<_>>()
                .join(" ");
            if let Some(cwd) = cwd.as_deref().filter(|cwd| !cwd.is_empty()) {
                format!("cd -- {} && {command}", shell_words::quote(cwd))
            } else {
                command
            }
        }
        DirectSshZmodemDownloadSource::DetectInteractiveSz => REMOTE_SZ_DETECT_SCRIPT.to_string(),
    }
}

const REMOTE_SZ_DETECT_SCRIPT: &str = r#"for p in /proc/[0-9]*; do [ "$(basename "$(readlink "$p/exe" 2>/dev/null)" 2>/dev/null)" = sz ] || continue; cwd=$(readlink "$p/cwd" 2>/dev/null) || continue; [ -r "$p/cmdline" ] || continue; cd -- "$cwd" || exit 1; exec xargs -0 -a "$p/cmdline" sh -c 'exec "$@"' sh; done; printf '%s\n' 'no active remote sz process found' >&2; exit 1"#;

pub fn legacy_ssh_zmodem_upload_timeout() -> Duration {
    SSH_ZMODEM_TIMEOUT
}

#[cfg(test)]
pub fn test_remote_rz_command(cwd: Option<&str>) -> String {
    remote_rz_command(cwd, false)
}

#[cfg(test)]
pub fn test_remote_rz_command_with_detection() -> String {
    remote_rz_command(None, true)
}

#[cfg(test)]
pub fn test_direct_ssh_args(
    host: &str,
    port: Option<&str>,
    cwd: Option<&str>,
) -> anyhow::Result<Vec<String>> {
    let upload = DirectSshZmodemUpload::new(
        InteractiveSshCommand {
            host: Some(host.to_string()),
            port: port.map(ToOwned::to_owned),
        },
        None,
        None,
        cwd.map(ToOwned::to_owned),
        false,
        ZmodemTransferPaths::upload(vec![PathBuf::from("file.txt")]),
        None,
    )?;
    direct_ssh_args(&upload)
}

#[cfg(test)]
pub fn test_direct_ssh_raw_upload_args(
    host: &str,
    cwd: Option<&str>,
) -> anyhow::Result<Vec<String>> {
    let upload = DirectSshZmodemUpload::new(
        InteractiveSshCommand {
            host: Some(host.to_string()),
            port: None,
        },
        None,
        None,
        cwd.map(ToOwned::to_owned),
        false,
        ZmodemTransferPaths::upload(vec![PathBuf::from("file.txt")]),
        None,
    )?;
    let files = vec![RawUploadFileSpec {
        path: PathBuf::from("file.txt"),
        name: String::from("file.txt"),
        size: 1234,
    }];
    let remote_command = remote_raw_upload_command(upload.cwd.as_deref(), false, None, &files);
    direct_ssh_args_for_remote_command(
        &upload.connection_info,
        upload.ssh_command.as_deref(),
        &remote_command,
        upload.auth_mode(),
    )
}

#[cfg(test)]
pub fn test_controlmaster_raw_upload_command(cwd: Option<&str>) -> String {
    let files = vec![RawUploadFileSpec {
        path: PathBuf::from("file.txt"),
        name: String::from("file.txt"),
        size: 1234,
    }];
    remote_raw_upload_command(cwd, false, None, &files)
}

#[cfg(test)]
pub fn test_direct_ssh_raw_upload_args_with_remote_rz_token(
    host: &str,
    token: &str,
) -> anyhow::Result<Vec<String>> {
    let upload = DirectSshZmodemUpload::new(
        InteractiveSshCommand {
            host: Some(host.to_string()),
            port: None,
        },
        None,
        None,
        None,
        true,
        ZmodemTransferPaths::upload(vec![PathBuf::from("file.txt")]),
        None,
    )?
    .with_remote_rz_token(token.to_string());
    let files = vec![RawUploadFileSpec {
        path: PathBuf::from("file.txt"),
        name: String::from("file.txt"),
        size: 1234,
    }];
    let remote_command = remote_raw_upload_command(
        upload.cwd.as_deref(),
        upload.detect_remote_rz_cwd,
        upload.remote_rz_token.as_deref(),
        &files,
    );
    direct_ssh_args_for_remote_command(
        &upload.connection_info,
        upload.ssh_command.as_deref(),
        &remote_command,
        upload.auth_mode(),
    )
}

#[cfg(test)]
pub fn test_direct_ssh_raw_download_args_from_command(
    command: &str,
    cwd: Option<&str>,
    argv: Vec<&str>,
) -> Option<Vec<String>> {
    let download = DirectSshZmodemDownload::new(
        InteractiveSshCommand {
            host: Some("user@example.com".to_string()),
            port: None,
        },
        Some(command.to_string()),
        None,
        DirectSshZmodemDownloadSource::Command {
            cwd: cwd.map(ToOwned::to_owned),
            argv: argv.into_iter().map(ToOwned::to_owned).collect(),
        },
        ZmodemTransferPaths::download_directory(PathBuf::from("H:\\Downloads")),
        None,
    )
    .ok()?;
    let remote_command = remote_raw_download_command(&download.source);
    direct_ssh_args_for_remote_command(
        &download.connection_info,
        download.ssh_command.as_deref(),
        &remote_command,
        direct_ssh_auth_mode(download.auth.as_ref()),
    )
    .ok()
}

#[cfg(test)]
pub fn test_sanitize_stderr(bytes: &[u8]) -> Option<String> {
    sanitize_stderr(bytes)
}

#[cfg(test)]
pub fn test_timeout_message_without_stderr() -> String {
    remote_rz_timeout_message(None)
}

#[cfg(test)]
pub fn test_direct_ssh_args_from_command(command: &str, cwd: Option<&str>) -> Option<Vec<String>> {
    direct_ssh_args_from_interactive_command(command, cwd, false, DirectSshZmodemAuthMode::None)
}

#[cfg(test)]
pub fn test_direct_ssh_args_from_active_command(command: &str) -> Option<Vec<String>> {
    direct_ssh_args_from_interactive_command(command, None, true, DirectSshZmodemAuthMode::None)
}

#[cfg(test)]
pub fn test_direct_ssh_args_from_command_with_password_auth(
    command: &str,
    cwd: Option<&str>,
) -> Option<Vec<String>> {
    direct_ssh_args_from_interactive_command(command, cwd, false, DirectSshZmodemAuthMode::Password)
}

#[cfg(test)]
pub fn test_direct_ssh_args_with_password_auth(
    host: &str,
    cwd: Option<&str>,
) -> anyhow::Result<Vec<String>> {
    let upload = DirectSshZmodemUpload::new(
        InteractiveSshCommand {
            host: Some(host.to_string()),
            port: None,
        },
        None,
        Some(DirectSshZmodemAuth::Password(Zeroizing::new(
            "secret".to_string(),
        ))),
        cwd.map(ToOwned::to_owned),
        false,
        ZmodemTransferPaths::upload(vec![PathBuf::from("file.txt")]),
        None,
    )?;
    direct_ssh_args(&upload)
}

#[cfg(test)]
pub fn test_direct_ssh_download_args_from_command(
    command: &str,
    cwd: Option<&str>,
    argv: Vec<&str>,
) -> Option<Vec<String>> {
    let download = DirectSshZmodemDownload::new(
        InteractiveSshCommand {
            host: Some("user@example.com".to_string()),
            port: None,
        },
        Some(command.to_string()),
        None,
        DirectSshZmodemDownloadSource::Command {
            cwd: cwd.map(ToOwned::to_owned),
            argv: argv.into_iter().map(ToOwned::to_owned).collect(),
        },
        ZmodemTransferPaths::download_directory(PathBuf::from("H:\\Downloads")),
        None,
    )
    .ok()?;
    direct_ssh_download_args(&download).ok()
}

#[cfg(test)]
pub fn test_direct_ssh_download_args_detecting_interactive_sz(
    command: &str,
) -> Option<Vec<String>> {
    let download = DirectSshZmodemDownload::new(
        InteractiveSshCommand {
            host: Some("user@example.com".to_string()),
            port: None,
        },
        Some(command.to_string()),
        None,
        DirectSshZmodemDownloadSource::DetectInteractiveSz,
        ZmodemTransferPaths::download_directory(PathBuf::from("H:\\Downloads")),
        None,
    )
    .ok()?;
    direct_ssh_download_args(&download).ok()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        raw_upload_write_failure_message, test_controlmaster_raw_upload_command,
        test_direct_ssh_args, test_direct_ssh_args_from_active_command,
        test_direct_ssh_args_from_command, test_direct_ssh_args_from_command_with_password_auth,
        test_direct_ssh_args_with_password_auth,
        test_direct_ssh_download_args_detecting_interactive_sz,
        test_direct_ssh_download_args_from_command, test_direct_ssh_raw_download_args_from_command,
        test_direct_ssh_raw_upload_args, test_direct_ssh_raw_upload_args_with_remote_rz_token,
        test_remote_rz_command, test_remote_rz_command_with_detection, test_sanitize_stderr,
        test_timeout_message_without_stderr, RawUploadFileSpec,
    };
    use crate::terminal::zmodem::{ZmodemDirection, ZmodemEvent};

    #[test]
    fn remote_rz_command_runs_in_selected_cwd() {
        let default_command = test_remote_rz_command(None);
        assert!(default_command.starts_with("target=$(pwd) && "));
        assert_remote_rz_command_stages_then_commits(&default_command);

        let cwd_command = test_remote_rz_command(Some("/tmp/with spaces"));
        assert!(cwd_command.starts_with("target='/tmp/with spaces' && "));
        assert_remote_rz_command_stages_then_commits(&cwd_command);
    }

    #[test]
    fn remote_rz_command_only_uses_an_unambiguous_receiver_cwd() {
        let command = test_remote_rz_command_with_detection();

        assert!(command.starts_with("target=\"$("));
        assert!(command.contains(r#"/proc/[0-9]*"#));
        assert!(command.contains("cat \"$p/comm\""));
        assert!(command.contains("case \"$comm\" in rz|lrz"));
        assert!(command.contains("case \"$exe\" in rz|lrz"));
        assert!(command.contains("tr '\\000' '\\n'"));
        assert!(command.contains("cwd=$(readlink \"$p/cwd\""));
        assert!(command.contains("multiple active remote rz working directories"));
        assert!(command.contains("no active remote rz process found"));
        assert!(!command.contains("shell_pid"));
        assert!(!command.contains(r#"readlink "$p/fd/0""#));
        assert!(!command.contains(r#"/dev/pts/*"#));
        assert!(!command.contains("done; pwd"));
        assert_remote_rz_command_stages_then_commits(&command);
    }

    #[test]
    fn raw_upload_write_failure_includes_source_and_remote_stderr() {
        let spec = RawUploadFileSpec {
            path: PathBuf::from("H:\\Downloads\\payload.bin"),
            name: String::from("payload.bin"),
            size: 4096,
        };
        let error = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "simulated broken pipe");

        let message = raw_upload_write_failure_message(
            &spec,
            1024,
            &error,
            None,
            Some("WARP_ZMODEM_TARGET=/tmp\ndd: failed to write: No space left on device"),
        );

        assert!(message.contains("H:\\Downloads\\payload.bin"));
        assert!(message.contains("1024/4096 bytes"));
        assert!(message.contains("simulated broken pipe"));
        assert!(message.contains("WARP_ZMODEM_TARGET=/tmp"));
        assert!(message.contains("No space left on device"));
    }

    #[test]
    fn direct_ssh_args_run_remote_rz_without_tty() {
        let args = test_direct_ssh_args("user@example.com", Some("2222"), Some("/tmp/with spaces"))
            .unwrap();
        assert_eq!(
            &args[..args.len() - 1],
            vec![
                "-T",
                "-o",
                "BatchMode=yes",
                "-o",
                "ForwardX11=no",
                "-p",
                "2222",
                "user@example.com",
            ]
        );
        assert_eq!(
            args.last().unwrap(),
            &test_remote_rz_command(Some("/tmp/with spaces"))
        );
    }

    #[test]
    fn direct_ssh_args_from_original_command_preserves_login_options() {
        let args = test_direct_ssh_args_from_command(
                "command ssh -F C:/Users/lc/.ssh/config -J bastion -i C:/keys/id_ed25519 -p 2222 ak/crop-ak5070",
                Some("/tmp"),
            )
            .unwrap();
        assert_eq!(
            &args[..args.len() - 1],
            vec![
                "-F",
                "C:/Users/lc/.ssh/config",
                "-J",
                "bastion",
                "-i",
                "C:/keys/id_ed25519",
                "-p",
                "2222",
                "-T",
                "-o",
                "BatchMode=yes",
                "-o",
                "ForwardX11=no",
                "ak/crop-ak5070",
            ]
        );
        assert_eq!(args.last().unwrap(), &test_remote_rz_command(Some("/tmp")));
    }

    #[test]
    fn direct_ssh_args_from_active_command_detects_remote_cwd_instead_of_using_local_cwd() {
        let args = test_direct_ssh_args_from_active_command("ssh alkaid@alkaid-5070").unwrap();
        let remote_command = args.last().unwrap();

        assert!(!remote_command.contains("C:\\Users\\lc"));
        assert!(remote_command.contains(r#"/proc/[0-9]*"#));
        assert!(remote_command.contains("case \"$comm\" in rz|lrz"));
        assert!(remote_command.contains("tr '\\000' '\\n'"));
        assert!(remote_command.contains("multiple active remote rz working directories"));
        assert!(remote_command.contains("no active remote rz process found"));
        assert!(!remote_command.contains(r#"/dev/pts/*"#));
        assert!(!remote_command.contains("done; pwd"));
        assert_remote_rz_command_stages_then_commits(remote_command);
        assert_eq!(args[args.len() - 2], "alkaid@alkaid-5070");
    }

    #[test]
    fn direct_ssh_args_with_password_auth_allows_askpass() {
        let args =
            test_direct_ssh_args_with_password_auth("user@example.com", Some("/tmp")).unwrap();
        assert_eq!(
            &args[..args.len() - 1],
            vec![
                "-T",
                "-o",
                "BatchMode=no",
                "-o",
                "PreferredAuthentications=password",
                "-o",
                "KbdInteractiveAuthentication=no",
                "-o",
                "NumberOfPasswordPrompts=1",
                "-o",
                "ForwardX11=no",
                "user@example.com",
            ]
        );
        assert_eq!(args.last().unwrap(), &test_remote_rz_command(Some("/tmp")));
    }

    #[test]
    fn direct_ssh_args_from_original_command_with_password_auth_allows_askpass() {
        let args = test_direct_ssh_args_from_command_with_password_auth(
            "ssh -F C:/Users/lc/.ssh/config -p 2222 alkaid@alkaid-5070",
            Some("/tmp"),
        )
        .unwrap();
        assert_eq!(
            &args[..args.len() - 1],
            vec![
                "-F",
                "C:/Users/lc/.ssh/config",
                "-p",
                "2222",
                "-T",
                "-o",
                "BatchMode=no",
                "-o",
                "PreferredAuthentications=password",
                "-o",
                "KbdInteractiveAuthentication=no",
                "-o",
                "NumberOfPasswordPrompts=1",
                "-o",
                "ForwardX11=no",
                "alkaid@alkaid-5070",
            ]
        );
        assert_eq!(args.last().unwrap(), &test_remote_rz_command(Some("/tmp")));
    }

    fn assert_remote_rz_command_stages_then_commits(command: &str) {
        assert!(command.contains(r#"mktemp -d ".warp-zmodem.XXXXXX""#));
        assert!(command.contains(r#"tmp=$(cd -- "$tmp" && pwd) || exit 1; cleanup()"#));
        assert!(command.contains(r#"trap cleanup EXIT HUP INT TERM"#));
        assert!(command.contains(r#"cd -- "$tmp" && rz -q -y"#));
        assert!(command.contains("ZMODEM upload target already exists"));
        assert!(command.contains(r#"ln -- "$f" "$target/$base""#));
        assert!(command.contains(r#"rm -f -- "$f""#));
        assert!(!command.contains("mv -f"));
        assert!(command.contains(r#"rm -rf -- "$tmp""#));
    }

    #[test]
    fn direct_ssh_raw_upload_args_stage_then_commit_without_rz() {
        let args = test_direct_ssh_raw_upload_args("user@example.com", Some("/tmp")).unwrap();
        let remote_command = args.last().unwrap();

        assert!(remote_command.starts_with("target=/tmp && "));
        assert!(remote_command.contains(r#"mktemp -d ".warp-zmodem.XXXXXX""#));
        assert!(remote_command.contains(r#"trap cleanup EXIT HUP INT TERM"#));
        assert!(remote_command.contains(r#"tmp=$(cd -- "$tmp" && pwd) || exit 1; cleanup()"#));
        assert!(remote_command.contains("WARP_ZMODEM_TARGET=%s"));
        assert!(remote_command.contains("WARP_ZMODEM_READY"));
        assert!(remote_command.contains("WARP_ZMODEM_STAGED"));
        assert!(remote_command.contains("receive_file 1234 file.txt"));
        assert!(remote_command.contains("dd bs=1048576 count=\"$size\""));
        assert!(remote_command.contains("IFS= read -r marker"));
        assert!(remote_command.contains(r#"[ "$marker" = "WARP_ZMODEM_COMMIT" ]"#));
        assert!(remote_command.contains("ZMODEM upload target already exists"));
        assert!(remote_command.contains("ln -- \"$f\" \"$target/$base\""));
        assert!(remote_command.contains("rm -f -- \"$f\""));
        assert!(!remote_command.contains("mv -f"));
        assert!(remote_command.contains("rm -rf -- \"$tmp\""));
        assert!(
            remote_command.find("WARP_ZMODEM_COMMIT").unwrap()
                < remote_command
                    .find("ZMODEM upload target already exists")
                    .unwrap()
        );
        assert!(
            remote_command
                .find("ZMODEM upload target already exists")
                .unwrap()
                < remote_command.find("ln --").unwrap()
        );
        assert!(
            remote_command.find("WARP_ZMODEM_STAGED").unwrap()
                < remote_command.find("WARP_ZMODEM_COMMIT").unwrap()
        );
        assert!(!remote_command.contains("rz -q -y"));
    }

    #[test]
    fn direct_ssh_drag_upload_matches_only_the_token_bound_remote_rz() {
        let args = test_direct_ssh_raw_upload_args_with_remote_rz_token(
            "user@example.com",
            "0123456789abcdef0123456789abcdef",
        )
        .unwrap();
        let remote_command = args.last().unwrap();

        assert!(remote_command.contains("expected_token=0123456789abcdef0123456789abcdef"));
        assert!(remote_command.contains(r#"/proc/[0-9]*"#));
        assert!(remote_command.contains(r#""$p/environ""#));
        assert!(remote_command.contains(r#"grep -Fqx "WARP_ZMODEM_TOKEN=$expected_token""#));
        assert!(remote_command.contains(r#"attempt=$((attempt + 1))"#));
        assert!(remote_command.contains("no matching remote rz process found"));
        assert!(!remote_command.contains(r#"/dev/pts/*"#));
        assert!(remote_command.contains("WARP_ZMODEM_READY"));
        assert!(remote_command.contains("receive_file 1234 file.txt"));
    }

    #[test]
    fn controlmaster_raw_upload_command_uses_same_staged_commit_protocol() {
        let remote_command = test_controlmaster_raw_upload_command(Some("/tmp"));

        assert!(remote_command.starts_with("target=/tmp && "));
        assert!(remote_command.contains("WARP_ZMODEM_READY"));
        assert!(remote_command.contains("WARP_ZMODEM_STAGED"));
        assert!(remote_command.contains(r#"[ "$marker" = "WARP_ZMODEM_COMMIT" ]"#));
        assert!(remote_command.contains("ZMODEM upload target already exists"));
        assert!(remote_command.contains("ln -- \"$f\" \"$target/$base\""));
        assert!(remote_command.contains("rm -f -- \"$f\""));
        assert!(!remote_command.contains("mv -f"));
        assert!(remote_command.contains("rm -rf -- \"$tmp\""));
        assert!(
            remote_command.find("WARP_ZMODEM_STAGED").unwrap()
                < remote_command.find("WARP_ZMODEM_COMMIT").unwrap()
        );
        assert!(
            remote_command.find("WARP_ZMODEM_COMMIT").unwrap()
                < remote_command
                    .find("ZMODEM upload target already exists")
                    .unwrap()
        );
        assert!(
            remote_command
                .find("ZMODEM upload target already exists")
                .unwrap()
                < remote_command.find("ln --").unwrap()
        );
        assert!(!remote_command.contains("rz -q -y"));
    }

    #[test]
    fn raw_upload_file_specs_rejects_duplicate_wire_names() {
        let dir = tempfile::tempdir().unwrap();
        let first_dir = dir.path().join("a");
        let second_dir = dir.path().join("b");
        std::fs::create_dir_all(&first_dir).unwrap();
        std::fs::create_dir_all(&second_dir).unwrap();
        let first_path = first_dir.join("foo.txt");
        let second_path = second_dir.join("foo.txt");
        std::fs::write(&first_path, b"first").unwrap();
        std::fs::write(&second_path, b"second").unwrap();

        let err =
            super::raw_upload_file_specs(&[first_path.clone(), second_path.clone()]).unwrap_err();
        let err = err.to_string();

        assert!(err.contains("multiple files named foo.txt"), "{err}");
        assert!(err.contains(&first_path.display().to_string()), "{err}");
        assert!(err.contains(&second_path.display().to_string()), "{err}");
    }

    #[test]
    fn direct_ssh_download_args_rerun_remote_sz_in_selected_remote_cwd() {
        assert_eq!(
            test_direct_ssh_download_args_from_command(
                "ssh -F C:/Users/lc/.ssh/config -p 2222 alkaid@alkaid-5070",
                Some("/home/alkaid/tmp"),
                vec!["sz", "tmp/codex-zmodem-test.txt"],
            )
            .unwrap(),
            vec![
                "-F",
                "C:/Users/lc/.ssh/config",
                "-p",
                "2222",
                "-T",
                "-o",
                "BatchMode=yes",
                "-o",
                "ForwardX11=no",
                "alkaid@alkaid-5070",
                "cd -- /home/alkaid/tmp && sz tmp/codex-zmodem-test.txt",
            ]
        );
    }

    #[test]
    fn direct_ssh_raw_download_args_stream_remote_files_without_sz() {
        let args = test_direct_ssh_raw_download_args_from_command(
            "ssh -F C:/Users/lc/.ssh/config -p 2222 alkaid@alkaid-5070",
            Some("/home/alkaid/tmp"),
            vec!["sz", "tmp/codex-zmodem-test.txt"],
        )
        .unwrap();
        let remote_command = args.last().unwrap();

        assert!(remote_command.starts_with("cd -- /home/alkaid/tmp && "));
        assert!(remote_command.contains("set -- tmp/codex-zmodem-test.txt"));
        assert!(remote_command.contains("printf 'file %s %s\\n'"));
        assert!(remote_command.contains(r#"name_len=$(printf '%s' "$base" | wc -c)"#));
        assert!(remote_command.contains("cat -- \"$f\""));
        assert!(remote_command.contains("printf 'done\\n'"));
        assert!(!remote_command.contains("sz tmp/codex-zmodem-test.txt"));
    }

    #[test]
    fn direct_ssh_download_args_can_detect_interactive_sz_process() {
        let args = test_direct_ssh_download_args_detecting_interactive_sz("ssh alkaid@alkaid-5070")
            .unwrap();
        let remote_command = args.last().unwrap();

        assert!(remote_command.contains(r#"/proc/[0-9]*"#));
        assert!(remote_command.contains(r#"= sz"#));
        assert!(remote_command.contains(r#"xargs -0 -a "$p/cmdline""#));
        assert_eq!(args[args.len() - 2], "alkaid@alkaid-5070");
    }

    #[test]
    fn raw_download_parser_persists_only_after_done() {
        let dir = tempfile::tempdir().unwrap();
        let mut parser = super::RawDownloadParser::new(dir.path().to_path_buf()).unwrap();
        let mut events = Vec::new();

        parser
            .append(b"file 5 10\nremote.txthello", &mut |event| {
                events.push(event)
            })
            .unwrap();

        assert!(!dir.path().join("remote.txt").exists());
        assert_eq!(zmodem_temp_paths(dir.path()).len(), 1);
        assert!(!events.iter().any(|event| matches!(
            event,
            ZmodemEvent::FileCompleted {
                direction: ZmodemDirection::Download,
                ..
            }
        )));

        parser
            .append(b"done\n", &mut |event| events.push(event))
            .unwrap();
        assert!(parser.completed);
        assert_eq!(
            parser
                .persist_completed_files(&mut |event| events.push(event))
                .unwrap(),
            5
        );
        assert_eq!(
            std::fs::read(dir.path().join("remote.txt")).unwrap(),
            b"hello"
        );
        assert!(zmodem_temp_paths(dir.path()).is_empty());
        assert!(events.iter().any(|event| {
            matches!(
                event,
                ZmodemEvent::FileCompleted {
                    direction: ZmodemDirection::Download,
                    name,
                    path: Some(path),
                } if name == "remote.txt" && path.exists()
            )
        }));
    }

    #[test]
    fn raw_download_parser_drop_removes_completed_temp_before_commit() {
        let dir = tempfile::tempdir().unwrap();
        let temp_path = {
            let mut parser = super::RawDownloadParser::new(dir.path().to_path_buf()).unwrap();
            parser
                .append(b"file 5 10\nremote.txthello", &mut |_| {})
                .unwrap();
            zmodem_temp_paths(dir.path())
                .into_iter()
                .next()
                .expect("completed file should still be staged")
        };

        assert!(!dir.path().join("remote.txt").exists());
        assert!(!temp_path.exists());
    }

    #[test]
    fn raw_download_parser_drop_removes_partial_temp_before_commit() {
        let dir = tempfile::tempdir().unwrap();
        let temp_path = {
            let mut parser = super::RawDownloadParser::new(dir.path().to_path_buf()).unwrap();
            parser
                .append(b"file 12 10\nremote.txtpartial", &mut |_| {})
                .unwrap();
            zmodem_temp_paths(dir.path())
                .into_iter()
                .next()
                .expect("partial file should be staged")
        };

        assert!(!dir.path().join("remote.txt").exists());
        assert!(!temp_path.exists());
    }

    #[test]
    fn raw_download_parser_reserves_completed_names_before_commit() {
        let dir = tempfile::tempdir().unwrap();
        let mut parser = super::RawDownloadParser::new(dir.path().to_path_buf()).unwrap();

        parser
            .append(b"file 3 10\nremote.txtone", &mut |_| {})
            .unwrap();
        parser
            .append(b"file 3 10\nremote.txttwo", &mut |_| {})
            .unwrap();
        parser.append(b"done\n", &mut |_| {}).unwrap();
        parser.persist_completed_files(&mut |_| {}).unwrap();

        assert_eq!(
            std::fs::read(dir.path().join("remote.txt")).unwrap(),
            b"one"
        );
        assert_eq!(
            std::fs::read(dir.path().join("remote (1).txt")).unwrap(),
            b"two"
        );
    }

    #[test]
    fn raw_download_parser_rejects_oversized_header() {
        let dir = tempfile::tempdir().unwrap();
        let mut parser = super::RawDownloadParser::new(dir.path().to_path_buf()).unwrap();
        let oversized_header = vec![b'a'; super::SSH_ZMODEM_RAW_DOWNLOAD_MAX_HEADER_LEN + 1];

        let err = parser.append(&oversized_header, &mut |_| {}).unwrap_err();

        assert!(err
            .to_string()
            .contains("raw ZMODEM download header exceeded"));
        assert!(zmodem_temp_paths(dir.path()).is_empty());
    }

    #[test]
    fn raw_download_parser_rejects_oversized_name_length() {
        let dir = tempfile::tempdir().unwrap();
        let mut parser = super::RawDownloadParser::new(dir.path().to_path_buf()).unwrap();
        let header = format!(
            "file 1 {}\n",
            super::SSH_ZMODEM_RAW_DOWNLOAD_MAX_NAME_LEN + 1
        );

        let err = parser.append(header.as_bytes(), &mut |_| {}).unwrap_err();

        assert!(err
            .to_string()
            .contains("raw ZMODEM download file name exceeded"));
        assert!(zmodem_temp_paths(dir.path()).is_empty());
    }

    #[test]
    fn raw_download_parser_rejects_oversized_file() {
        let dir = tempfile::tempdir().unwrap();
        let mut parser = super::RawDownloadParser::new(dir.path().to_path_buf()).unwrap();
        let header = format!(
            "file {} 10\n",
            super::SSH_ZMODEM_RAW_DOWNLOAD_MAX_FILE_SIZE + 1
        );

        let err = parser.append(header.as_bytes(), &mut |_| {}).unwrap_err();

        assert!(err
            .to_string()
            .contains("raw ZMODEM download file exceeded"));
        assert!(zmodem_temp_paths(dir.path()).is_empty());
    }

    #[test]
    fn raw_download_parser_rejects_oversized_total() {
        let err = super::validate_raw_download_limits(
            2,
            8,
            super::SSH_ZMODEM_RAW_DOWNLOAD_MAX_TOTAL_SIZE - 1,
        )
        .unwrap_err();

        assert!(err
            .to_string()
            .contains("raw ZMODEM download total size exceeded"));
    }

    #[test]
    fn sanitize_stderr_trims_noise_without_dropping_message() {
        assert_eq!(
            test_sanitize_stderr(b"\r\nssh: Could not resolve hostname alkaid-5070\r\n").unwrap(),
            "ssh: Could not resolve hostname alkaid-5070"
        );
        assert_eq!(test_sanitize_stderr(b"\r\n\0\t"), None);
    }

    #[test]
    fn timeout_message_names_remote_rz_initialization() {
        assert_eq!(
            test_timeout_message_without_stderr(),
            "remote rz did not send ZMODEM initialization within 15s"
        );
    }

    fn zmodem_temp_paths(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
        std::fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| {
                        name.starts_with(".warp-zmodem-") && name.ends_with(".part")
                    })
            })
            .collect()
    }
}
