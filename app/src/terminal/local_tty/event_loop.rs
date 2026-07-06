//! The main event loop which performs I/O on the pseudoterminal.

use std::{
    borrow::Cow,
    collections::VecDeque,
    io::{self, ErrorKind, Read, Write},
    marker::Send,
    ops::DerefMut,
    sync::Arc,
    thread::{self, JoinHandle},
};

use log::error;
use mio::{self, Events, Interest};
use parking_lot::{FairMutex, FairMutexGuard};

use crate::terminal::{
    TerminalModel, event_listener::ChannelEventListener, local_tty, model::ansi,
};
use crate::terminal::{
    event::Event as TerminalEvent,
    model::terminal_model::ExitReason,
    writeable_pty::Message,
    zmodem::{
        PendingZmodemSession, ZmodemDetector, ZmodemDetectorResult, ZmodemDirection, ZmodemEvent,
        ZmodemSession, ZmodemTransferPaths, zmodem_error_event,
    },
};

use super::mio_channel::Receiver;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WriteSource {
    UserInput,
    Zmodem,
}

struct PendingWrite {
    source: WriteSource,
    bytes: Cow<'static, [u8]>,
}

/// The size of the buffer to read data into from the PTY.
const READ_BUFFER_SIZE: usize = 0x4_0000;

/// Max bytes to process from the PTY while holding the lock before giving
/// someone else an opportunity to lock it.
const MAX_LOCKED_READ: usize = 0x1_0000;

pub const CHANNEL_TOKEN: mio::Token = mio::Token(0);
pub const PTY_TOKEN: mio::Token = mio::Token(1);
pub const SIGNALS_TOKEN: mio::Token = mio::Token(2);

/// The main event!.. loop.
///
/// Handles all the PTY I/O and runs the PTY parser which updates terminal
/// state.
pub struct EventLoop<T: local_tty::EventedPty> {
    poll: mio::Poll,
    pty: T,
    rx: Receiver<Message>,
    terminal: Arc<FairMutex<TerminalModel>>,

    /// The event listener is available to the PTY event loop
    /// to emit relevant events to subscribers. The ansi handler
    /// also has a handle to the event listener, so events may also
    /// be emitted at a later stage (i.e. when we have a better idea
    /// of what the bytes from the PTY actually meant).
    event_listener: ChannelEventListener,
}

/// Helper type which tracks how much of a buffer has been written.
struct Writing {
    source: WriteSource,
    bytes: Cow<'static, [u8]>,
    written: usize,
}

/// All of the mutable state needed to run the event loop.
///
/// Contains list of items to write, current write state, etc. Anything that
/// would otherwise be mutated on the `EventLoop` goes here.
pub struct State {
    write_list: VecDeque<PendingWrite>,
    writing: Option<Writing>,
    parser: ansi::Processor,
    zmodem: ZmodemState,
    zmodem_detector: ZmodemDetector,
}

impl Default for State {
    fn default() -> State {
        State {
            write_list: VecDeque::new(),
            parser: ansi::Processor::new(),
            writing: None,
            zmodem: ZmodemState::Inactive,
            zmodem_detector: ZmodemDetector::default(),
        }
    }
}

enum ZmodemState {
    Inactive,
    Pending(PendingZmodemSession),
    Active(ZmodemSession),
}

impl State {
    #[inline]
    fn ensure_next(&mut self) {
        if self.writing.is_none() {
            self.goto_next();
        }
    }

    #[inline]
    fn goto_next(&mut self) {
        self.writing = self.write_list.pop_front().map(Writing::new);
    }

    #[inline]
    fn take_current(&mut self) -> Option<Writing> {
        self.writing.take()
    }

    #[inline]
    fn needs_write(&self) -> bool {
        self.writing.is_some() || !self.write_list.is_empty()
    }

    #[inline]
    fn set_current(&mut self, new: Option<Writing>) {
        self.writing = new;
    }
}

impl Writing {
    #[inline]
    fn new(pending: PendingWrite) -> Writing {
        Writing {
            source: pending.source,
            bytes: pending.bytes,
            written: 0,
        }
    }

    #[inline]
    fn advance(&mut self, n: usize) {
        self.written += n;
    }

    #[inline]
    fn remaining_bytes(&self) -> &[u8] {
        &self.bytes[self.written..]
    }

    #[inline]
    fn finished(&self) -> bool {
        self.written >= self.bytes.len()
    }

    #[inline]
    fn len(&self) -> usize {
        self.bytes.len()
    }
}

enum ChannelResult {
    Continue { should_try_write: bool },
    TerminateLoop { child_exited: bool },
}

impl<T> EventLoop<T>
where
    T: local_tty::EventedPty + Send + 'static,
{
    /// Create a new event loop.
    pub fn new(
        terminal: Arc<FairMutex<TerminalModel>>,
        event_listener: ChannelEventListener,
        pty: T,
        rx: Receiver<Message>,
    ) -> EventLoop<T> {
        EventLoop {
            poll: mio::Poll::new().expect("create mio Poll"),
            pty,
            rx,
            terminal,
            event_listener,
        }
    }

    /// Drain the channel.
    ///
    /// Returns `false` when a shutdown message was received.
    fn drain_recv_channel(&mut self, state: &mut State) -> ChannelResult {
        let mut should_try_write = false;
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                Message::Input(input) => {
                    state.write_list.push_back(PendingWrite {
                        source: WriteSource::UserInput,
                        bytes: input,
                    });
                    should_try_write = true;
                }
                Message::ZmodemTransferPaths(paths) => {
                    self.handle_zmodem_paths(state, paths);
                    should_try_write = true;
                }
                Message::AbortZmodemSilently => {
                    Self::abort_zmodem_silently(state);
                    should_try_write = true;
                }
                Message::Shutdown => {
                    return ChannelResult::TerminateLoop {
                        child_exited: false,
                    };
                }
                Message::Resize(size) => self.pty.on_resize(&size),
                Message::ChildExited => return ChannelResult::TerminateLoop { child_exited: true },
            }
        }

        ChannelResult::Continue { should_try_write }
    }

    /// Returns a `bool` indicating whether or not the event loop should continue running.
    #[inline]
    fn channel_event(&mut self, state: &mut State) -> ChannelResult {
        self.drain_recv_channel(state)
    }

    fn emit_zmodem_event(&self, event: ZmodemEvent) {
        self.event_listener
            .send_terminal_event(TerminalEvent::Zmodem(event));
    }

    fn queue_zmodem_wire(state: &mut State, bytes: Vec<u8>) {
        if !bytes.is_empty() {
            log::debug!("ZMODEM queueing {} byte(s) for PTY write", bytes.len());
            state.write_list.push_back(PendingWrite {
                source: WriteSource::Zmodem,
                bytes: Cow::Owned(bytes),
            });
        }
    }

    fn discard_queued_zmodem_writes(state: &mut State) {
        state
            .write_list
            .retain(|pending| pending.source != WriteSource::Zmodem);
        if state
            .writing
            .as_ref()
            .is_some_and(|writing| writing.source == WriteSource::Zmodem)
        {
            state.writing = None;
        }
    }

    fn abort_zmodem_silently(state: &mut State) {
        let previous = std::mem::replace(&mut state.zmodem, ZmodemState::Inactive);
        match previous {
            ZmodemState::Pending(pending) => {
                log::debug!(
                    "ZMODEM silently aborting pending {:?} transfer",
                    pending.direction()
                );
                Self::discard_queued_zmodem_writes(state);
                Self::queue_zmodem_wire(state, pending.cancel());
            }
            ZmodemState::Active(session) => {
                let direction = session.direction();
                log::debug!("ZMODEM silently aborting active {direction:?} transfer");
                Self::discard_queued_zmodem_writes(state);
                Self::queue_zmodem_wire(state, session.cancel());
            }
            ZmodemState::Inactive => {}
        }
        state.zmodem_detector.take_pending_ordinary();
    }

    fn fail_zmodem(
        &self,
        state: &mut State,
        direction: Option<ZmodemDirection>,
        err: impl std::fmt::Display,
    ) {
        self.emit_zmodem_event(zmodem_error_event(direction, err));
        state.zmodem = ZmodemState::Inactive;
    }

    fn handle_zmodem_paths(&mut self, state: &mut State, paths: ZmodemTransferPaths) {
        log::info!(
            "PTY event loop received ZMODEM {:?} path selection: path_count={}",
            paths.direction,
            paths.paths.len()
        );
        let previous = std::mem::replace(&mut state.zmodem, ZmodemState::Inactive);
        let pending = match previous {
            ZmodemState::Pending(pending) => pending,
            ZmodemState::Active(session) if paths.paths.is_empty() => {
                let direction = session.direction();
                Self::discard_queued_zmodem_writes(state);
                Self::queue_zmodem_wire(state, session.cancel());
                log::debug!("ZMODEM cancelling active {direction:?} transfer");
                self.emit_zmodem_event(ZmodemEvent::Cancelled { direction });
                return;
            }
            other => {
                state.zmodem = other;
                if paths.paths.is_empty() {
                    self.emit_zmodem_event(ZmodemEvent::Cancelled {
                        direction: paths.direction,
                    });
                    return;
                }
                self.emit_zmodem_event(ZmodemEvent::Failed {
                    direction: Some(paths.direction),
                    message: "ZMODEM session is no longer active".to_string(),
                });
                return;
            }
        };

        if paths.paths.is_empty() {
            let direction = pending.direction();
            Self::discard_queued_zmodem_writes(state);
            Self::queue_zmodem_wire(state, pending.cancel());
            log::debug!("ZMODEM cancelling pending {direction:?} transfer");
            self.emit_zmodem_event(ZmodemEvent::Cancelled { direction });
            return;
        }

        let direction = pending.direction();
        match pending.start(paths) {
            Ok(session) => {
                log::info!("ZMODEM starting {direction:?} transfer from selected paths");
                state.zmodem = ZmodemState::Active(session);
                let trailing_output = self.drain_zmodem_actions(state);
                let ordinary_output =
                    Self::process_pty_bytes(&self.event_listener, state, &trailing_output);
                self.parse_ordinary_output(state, &ordinary_output);
            }
            Err(err) => self.fail_zmodem(state, Some(direction), err),
        }
    }

    fn drain_zmodem_actions(&self, state: &mut State) -> Vec<u8> {
        let (direction, wire_bytes, events, result) = {
            let ZmodemState::Active(session) = &mut state.zmodem else {
                return Vec::new();
            };
            let direction = session.direction();
            let mut wire_bytes = Vec::new();
            let mut events = Vec::new();
            let result =
                session.drain_actions(|bytes| wire_bytes.extend(bytes), |event| events.push(event));
            (direction, wire_bytes, events, result)
        };

        Self::queue_zmodem_wire(state, wire_bytes);
        for event in events {
            self.emit_zmodem_event(event);
        }
        match result {
            Ok(true) => {
                let trailing_output = match &mut state.zmodem {
                    ZmodemState::Active(session) => session.take_input(),
                    ZmodemState::Inactive | ZmodemState::Pending(_) => Vec::new(),
                };
                state.zmodem = ZmodemState::Inactive;
                return trailing_output;
            }
            Ok(false) => {}
            Err(err) => self.fail_zmodem(state, Some(direction), err),
        }
        Vec::new()
    }

    fn process_pty_bytes(
        event_listener: &ChannelEventListener,
        state: &mut State,
        bytes: &[u8],
    ) -> Vec<u8> {
        match &mut state.zmodem {
            ZmodemState::Inactive => {
                if warp_core::features::FeatureFlag::Lrzsz.is_enabled() {
                    match state.zmodem_detector.push(bytes) {
                        ZmodemDetectorResult::Detected {
                            detection,
                            ordinary_output,
                            zmodem_input,
                        } => {
                            log::debug!(
                                "ZMODEM detected {:?} session: zmodem_input_len={}, ordinary_len={}",
                                detection.direction,
                                zmodem_input.len(),
                                ordinary_output.len()
                            );
                            state.zmodem = ZmodemState::Pending(PendingZmodemSession::new(
                                detection.direction,
                                &zmodem_input,
                            ));
                            let event = match detection.direction {
                                ZmodemDirection::Upload => ZmodemEvent::UploadRequested,
                                ZmodemDirection::Download => {
                                    ZmodemEvent::DownloadDirectoryRequested
                                }
                            };
                            event_listener.send_terminal_event(TerminalEvent::Zmodem(event));
                            return ordinary_output;
                        }
                        ZmodemDetectorResult::Ordinary(ordinary_output) => {
                            return ordinary_output;
                        }
                    }
                }
                let mut ordinary_output = state.zmodem_detector.take_pending_ordinary();
                ordinary_output.extend_from_slice(bytes);
                ordinary_output
            }
            ZmodemState::Pending(pending) => {
                log::debug!(
                    "ZMODEM buffering pending {:?} input: len={}",
                    pending.direction(),
                    bytes.len()
                );
                pending.append_input(bytes);
                Vec::new()
            }
            ZmodemState::Active(session) => {
                let (direction, wire_bytes, events, result) = {
                    session.append_input(bytes);
                    let direction = session.direction();
                    log::debug!(
                        "ZMODEM active {direction:?} received PTY bytes: len={}",
                        bytes.len()
                    );
                    let mut wire_bytes = Vec::new();
                    let mut events = Vec::new();
                    let result = session.drain_actions(
                        |bytes| wire_bytes.extend(bytes),
                        |event| events.push(event),
                    );
                    (direction, wire_bytes, events, result)
                };
                Self::queue_zmodem_wire(state, wire_bytes);
                for event in events {
                    event_listener.send_terminal_event(TerminalEvent::Zmodem(event));
                }
                match result {
                    Ok(true) => {
                        let trailing_output = match &mut state.zmodem {
                            ZmodemState::Active(session) => session.take_input(),
                            ZmodemState::Inactive | ZmodemState::Pending(_) => Vec::new(),
                        };
                        state.zmodem = ZmodemState::Inactive;
                        return Self::process_pty_bytes(event_listener, state, &trailing_output);
                    }
                    Ok(false) => {}
                    Err(err) => {
                        event_listener.send_terminal_event(TerminalEvent::Zmodem(
                            zmodem_error_event(Some(direction), err),
                        ));
                        state.zmodem = ZmodemState::Inactive;
                    }
                }
                Vec::new()
            }
        }
    }

    fn parse_ordinary_output(&mut self, state: &mut State, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }

        let mut terminal = self.terminal.lock();
        state
            .parser
            .parse_bytes(terminal.deref_mut(), bytes, self.pty.writer());
        self.event_listener.send_wakeup_event();
    }

    /// Reads from the pty into the provided buffer, using the provided state
    /// information in order to properly advance the ANSI parser.
    ///
    /// If `writer` is `Some`, a copy of all bytes read will be written to that
    /// writer.
    ///
    /// Returns the number of bytes read from the PTY.
    #[inline]
    #[allow(clippy::unwrap_in_result)]
    fn pty_read(
        &mut self,
        state: &mut State,
        buf: &mut [u8],
        can_read: &mut bool,
    ) -> io::Result<()> {
        let mut bytes_in_buffer = 0;
        let mut ordinary_output = Vec::new();
        let mut bytes_processed = 0;

        // We read up to sizeof(buf) to limit the amount of time spent
        // reading from the PTY for a given event. Currently, the buf
        // has size [`MAX_READ`].
        loop {
            match self.pty.reader().read(&mut buf[bytes_in_buffer..]) {
                Ok(0) if bytes_in_buffer == 0 => {
                    // If we get 0 here with an empty buffer (guaranteed if
                    // bytes_in_buffer == 0), it means the object is unable to
                    // receive reads.
                    *can_read = false;
                    // There is nothing to be processed in the buffer, so return
                    // to the event loop.
                    break;
                }
                // Otherwise, track how many additional bytes we read and move
                // on to byte processing.
                Ok(got) => bytes_in_buffer += got,
                Err(err) => match err.kind() {
                    ErrorKind::Interrupted | ErrorKind::WouldBlock => {
                        if err.kind() == ErrorKind::WouldBlock {
                            *can_read = false;
                        }
                        if bytes_in_buffer == 0 {
                            break;
                        }
                    }
                    _ => return Err(err),
                },
            }

            let parsed_output =
                Self::process_pty_bytes(&self.event_listener, state, &buf[..bytes_in_buffer]);
            ordinary_output.extend(parsed_output);
            bytes_in_buffer = 0;

            if !ordinary_output.is_empty() {
                let should_block_for_lock = ordinary_output.len() >= READ_BUFFER_SIZE;
                let terminal = if should_block_for_lock {
                    Some(self.terminal.lock())
                } else {
                    self.terminal.try_lock()
                };

                let Some(mut terminal) = terminal else {
                    continue;
                };

                let parsed_len = ordinary_output.len();
                state
                    .parser
                    .parse_bytes(terminal.deref_mut(), &ordinary_output, self.pty.writer());
                ordinary_output.clear();
                bytes_processed += parsed_len;
                FairMutexGuard::bump(&mut terminal);
            }

            if bytes_processed >= MAX_LOCKED_READ {
                break;
            }
        }

        if !ordinary_output.is_empty() {
            let parsed_len = ordinary_output.len();
            let mut terminal = self.terminal.lock();
            state
                .parser
                .parse_bytes(terminal.deref_mut(), &ordinary_output, self.pty.writer());
            bytes_processed += parsed_len;
        }

        // Queue a terminal redraw if we processed some number
        // of non-(synchronized output) bytes.
        if bytes_processed > state.parser.sync_output_buffer_len().unwrap_or(0) {
            self.event_listener.send_wakeup_event();
        }

        Ok(())
    }

    #[inline]
    fn pty_write(&mut self, state: &mut State, can_write: &mut bool) -> io::Result<()> {
        state.ensure_next();

        'write_many: while let Some(mut current) = state.take_current() {
            'write_one: loop {
                match self.pty.writer().write(current.remaining_bytes()) {
                    Ok(0) => {
                        if current.source == WriteSource::Zmodem {
                            log::debug!(
                                "ZMODEM PTY write accepted 0 byte(s) after {}/{} byte(s)",
                                current.written,
                                current.len()
                            );
                        }
                        state.set_current(Some(current));
                        // We never attempt to write an empty buffer, so if we
                        // get 0 here, it means the object is unable to receive
                        // writes.
                        *can_write = false;
                        break 'write_many;
                    }
                    Ok(n) => {
                        if current.source == WriteSource::Zmodem {
                            log::debug!(
                                "ZMODEM wrote {n} byte(s) to PTY ({}/{} total)",
                                current.written + n,
                                current.len()
                            );
                        }
                        current.advance(n);
                        if current.finished() {
                            if current.source == WriteSource::Zmodem {
                                log::debug!(
                                    "ZMODEM finished writing {} byte(s) to PTY",
                                    current.len()
                                );
                            }
                            state.goto_next();
                            break 'write_one;
                        }
                    }
                    Err(err) => match err.kind() {
                        ErrorKind::Interrupted | ErrorKind::WouldBlock => {
                            if err.kind() == ErrorKind::WouldBlock {
                                if current.source == WriteSource::Zmodem {
                                    let written = current.written;
                                    let len = current.len();
                                    log::debug!(
                                        "ZMODEM PTY write would block after {}/{} byte(s)",
                                        written,
                                        len
                                    );
                                }
                                *can_write = false;
                            }
                            state.set_current(Some(current));
                            break 'write_many;
                        }
                        _ => {
                            state.set_current(Some(current));
                            return Err(err);
                        }
                    },
                }
            }
        }

        Ok(())
    }

    pub fn spawn(mut self) -> JoinHandle<()> {
        #[cfg(test)]
        let feature_flag_overrides = warp_core::features::get_overrides();

        thread::Builder::new()
            .name("PTY reader".into())
            .spawn(move || {
                // Make sure any overridden feature flags are also overridden
                // in the PTY reader thread.
                #[cfg(test)]
                warp_core::features::set_overrides(feature_flag_overrides);

                let mut state = State::default();
                let mut buf = [0u8; READ_BUFFER_SIZE];

                // Keep track of whether we've "drained" read and write
                // readiness.  Once we receive a read or write readiness event,
                // we won't receive another until the operation would block.
                // These let us know whether we should keep processing reads
                // and writes, even without receiving a new readiness event.
                let mut can_read = false;
                let mut can_write = false;

                self.poll
                    .registry()
                    .register(&mut self.rx, CHANNEL_TOKEN, Interest::READABLE)
                    .unwrap();

                // Register TTY through EventedRW interface.
                self.pty
                    .register(&self.poll, Interest::READABLE | Interest::WRITABLE)
                    .unwrap();

                let mut events = Events::with_capacity(1024);

                // True if the child exiting caused the event loop to wind down
                // (e.g. CTRL D or `exit`) rather than the inverse.
                let mut child_exited = false;

                'event_loop: loop {
                    // Clear the events so that we can reliably equate the absence of events
                    // to the timeout being fired.
                    events.clear();

                    // Wait for events, but only up to the remaining timeout for the synchronous output
                    // update (if any).
                    let sync_state_timeout = state.parser.sync_output_remaining_timeout();
                    if let Err(err) = self.poll.poll(&mut events, sync_state_timeout) {
                        match err.kind() {
                            ErrorKind::Interrupted => continue,
                            _ => panic!("EventLoop polling error: {err:?}"),
                        }
                    }

                    // If there were no events but `poll` returned, that means we hit the timeout.
                    if events.is_empty() {
                        state
                            .parser
                            .finish_sync_output(&mut *self.terminal.lock(), &mut self.pty.writer());
                        continue;
                    }

                    for event in events.iter() {
                        match event.token() {
                            token if token == CHANNEL_TOKEN => {
                                match self.channel_event(&mut state) {
                                    ChannelResult::Continue { should_try_write } => {
                                        if should_try_write && state.needs_write() {
                                            can_write = true;
                                        }
                                    }
                                    ChannelResult::TerminateLoop {
                                        child_exited: exited,
                                    } => {
                                        if exited {
                                            self.terminal
                                                .lock()
                                                .exit(ExitReason::ShellProcessExited);
                                            child_exited = true;
                                            self.event_listener.send_wakeup_event();
                                        }
                                        break 'event_loop;
                                    }
                                }
                            }

                            token if token == self.pty.child_event_token() => {
                                if let Some(local_tty::ChildEvent::Exited) =
                                    self.pty.next_child_event()
                                {
                                    self.terminal.lock().exit(ExitReason::ShellProcessExited);
                                    child_exited = true;
                                    self.event_listener.send_wakeup_event();
                                    break 'event_loop;
                                }
                            }

                            token
                                if token == self.pty.read_token()
                                    || token == self.pty.write_token() =>
                            {
                                #[cfg(unix)]
                                if event.is_read_closed() || event.is_write_closed() {
                                    // Don't try to do I/O on a dead PTY.
                                    continue;
                                }

                                if event.is_readable() {
                                    can_read = true;
                                }
                                if event.is_writable() {
                                    can_write = true;
                                }
                            }
                            _ => (),
                        }
                    }

                    // As long as we have work to do, do it.  Once we need to
                    // wait on some readiness (pty readability, pty writability,
                    // or new data to write), go back to the start of the event
                    // loop.
                    while can_read || (state.needs_write() && can_write) {
                        if can_read {
                            match self.pty_read(&mut state, &mut buf, &mut can_read) {
                                Ok(_) => {
                                    if state.needs_write() {
                                        can_write = true;
                                    }
                                }
                                Err(err) => {
                                    // On Linux, a `read` on the master side of a PTY can fail
                                    // with `EIO` if the client side hangs up.  In that case,
                                    // just loop back round for the inevitable `Exited` event.
                                    // This sucks, but checking the process is either racy or
                                    // blocking.
                                    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
                                    if err.kind() == ErrorKind::Other {
                                        continue;
                                    }

                                    error!("Error reading from PTY in event loop: {err}");
                                    break 'event_loop;
                                }
                            }
                        }

                        if state.needs_write() && can_write {
                            if let Err(err) = self.pty_write(&mut state, &mut can_write) {
                                error!("Error writing to PTY in event loop: {err}");
                                break 'event_loop;
                            }
                        }
                    }
                }

                // The evented instances are not dropped here so deregister them explicitly.
                let _ = self.poll.registry().deregister(&mut self.rx);
                let _ = self.pty.deregister(&self.poll);

                // Terminate the PTY process, if it's not the initiator of the shutdown.
                if !child_exited {
                    let res = self.pty.kill();
                    if let Err(err) = res {
                        log::error!("Failed to kill PTY process {err:?}");
                    }
                }
                // Notify the terminal model that the PTY process has exited.
                self.terminal.lock().exit(ExitReason::PtyDisconnected);
            })
            .expect("thread spawn works")
    }
}

#[cfg(test)]
#[path = "event_loop_tests.rs"]
mod tests;
