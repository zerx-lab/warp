use std::{io, sync::Arc};

use parking_lot::FairMutex;
use tempfile::tempdir;
use zmodem2::Action;

use super::super::mio_channel;
use super::{ChannelResult, EventLoop, State, ZmodemState};
use crate::terminal::event::Event as TerminalEvent;
use crate::terminal::event_listener::ChannelEventListener;
use crate::terminal::zmodem::{
    MockZmodemSession, PendingZmodemSession, ZmodemDirection, ZmodemEvent, ZmodemSession,
    ZmodemTransferPaths, ZMODEM_ABORT_SEQUENCE,
};
use crate::terminal::{local_tty, writeable_pty::Message, zmodem::ZmodemDetector, TerminalModel};

#[test]
fn zmodem_detection_is_ordinary_output_when_feature_disabled() {
    let _flag = warp_core::features::FeatureFlag::Lrzsz.override_enabled(false);
    let (events_tx, events_rx) = async_channel::unbounded();
    let listener = ChannelEventListener::builder_for_test()
        .with_terminal_events_tx(events_tx)
        .build();
    let mut state = State::default();

    let output =
        EventLoop::<MockPty>::process_pty_bytes(&listener, &mut state, b"ordinary**\x18B00payload");

    assert_eq!(output, b"ordinary**\x18B00payload");
    assert!(matches!(state.zmodem, ZmodemState::Inactive));
    assert!(events_rx.is_empty());
}

#[test]
fn zmodem_detection_emits_request_when_feature_enabled() {
    let _flag = warp_core::features::FeatureFlag::Lrzsz.override_enabled(true);
    let (events_tx, events_rx) = async_channel::unbounded();
    let listener = ChannelEventListener::builder_for_test()
        .with_terminal_events_tx(events_tx)
        .build();
    let mut state = State::default();

    let output = EventLoop::<MockPty>::process_pty_bytes(
        &listener,
        &mut state,
        b"ready\r\n**\x18B01000000000000",
    );

    assert_eq!(output, b"ready\r\n");
    assert!(matches!(state.zmodem, ZmodemState::Pending(_)));
    assert!(matches!(
        events_rx.try_recv().unwrap(),
        TerminalEvent::Zmodem(ZmodemEvent::UploadRequested)
    ));
}

#[test]
fn drag_started_rz_command_echo_is_hidden_before_zmodem_detection() {
    let _flag = warp_core::features::FeatureFlag::Lrzsz.override_enabled(true);
    let token = "0123456789abcdef0123456789abcdef";
    let command = format!("\x15env WARP_ZMODEM_TOKEN={token} rz\r");
    let command_echo = format!("env WARP_ZMODEM_TOKEN={token} rz");
    let (events_tx, events_rx) = async_channel::unbounded();
    let listener = ChannelEventListener::builder_for_test()
        .with_terminal_events_tx(events_tx)
        .build();
    let (tx, rx) = mio_channel::channel();
    tx.send(Message::Input(command.into_bytes().into()))
        .unwrap();
    let mut loop_ = EventLoop::new(
        Arc::new(FairMutex::new(TerminalModel::mock(None, None))),
        listener,
        MockPty,
        rx,
    );
    let mut state = State::default();

    loop_.drain_recv_channel(&mut state);
    let split = command_echo.len() / 2;
    let first = format!("alkaid@host:~/tmp$ {}", &command_echo[..split]);
    let second = format!("{}\r\n**\x18B01000000000000", &command_echo[split..]);
    let mut output = EventLoop::<MockPty>::process_pty_bytes(
        &loop_.event_listener,
        &mut state,
        first.as_bytes(),
    );
    output.extend(EventLoop::<MockPty>::process_pty_bytes(
        &loop_.event_listener,
        &mut state,
        second.as_bytes(),
    ));

    assert_eq!(output, b"alkaid@host:~/tmp$ \r\n");
    assert!(!String::from_utf8_lossy(&output).contains("WARP_ZMODEM_TOKEN"));
    assert!(state.zmodem_drag_command_echo.is_none());
    assert!(matches!(state.zmodem, ZmodemState::Pending(_)));
    assert!(matches!(
        events_rx.try_recv().unwrap(),
        TerminalEvent::Zmodem(ZmodemEvent::UploadRequested)
    ));
}

#[test]
fn completed_zmodem_session_reprocesses_trailing_ordinary_output() {
    let _flag = warp_core::features::FeatureFlag::Lrzsz.override_enabled(true);
    let (events_tx, events_rx) = async_channel::unbounded();
    let listener = ChannelEventListener::builder_for_test()
        .with_terminal_events_tx(events_tx)
        .build();
    let mut state = State {
        zmodem: ZmodemState::Active(ZmodemSession::Mock(MockZmodemSession {
            direction: ZmodemDirection::Download,
            input: b"prompt> ".to_vec(),
            events: vec![ZmodemEvent::Completed {
                direction: ZmodemDirection::Download,
            }],
            done: true,
        })),
        zmodem_detector: ZmodemDetector::default(),
        ..State::default()
    };

    let output = EventLoop::<MockPty>::process_pty_bytes(&listener, &mut state, b"");

    assert_eq!(output, b"prompt> ");
    assert!(matches!(state.zmodem, ZmodemState::Inactive));
    assert!(matches!(
        events_rx.try_recv().unwrap(),
        TerminalEvent::Zmodem(ZmodemEvent::Completed {
            direction: ZmodemDirection::Download
        })
    ));
}

#[test]
fn zmodem_paths_message_requests_write_readiness() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("upload.txt");
    std::fs::write(&source_path, b"hello").unwrap();
    let mut receiver = zmodem2::Receiver::new().unwrap();
    let receiver_init = match receiver.poll() {
        Action::WriteWire(bytes) => bytes.to_vec(),
        other => panic!("receiver should advertise ZRINIT first, got {other:?}"),
    };

    let (tx, rx) = mio_channel::channel();
    tx.send(Message::ZmodemTransferPaths(ZmodemTransferPaths::upload(
        vec![source_path],
    )))
    .unwrap();
    let mut loop_ = EventLoop::new(
        Arc::new(FairMutex::new(TerminalModel::mock(None, None))),
        ChannelEventListener::new_for_test(),
        MockPty,
        rx,
    );
    let mut state = State {
        zmodem: ZmodemState::Pending(PendingZmodemSession::new(
            ZmodemDirection::Upload,
            &receiver_init,
        )),
        ..State::default()
    };

    let result = loop_.drain_recv_channel(&mut state);

    assert!(matches!(
        result,
        ChannelResult::Continue {
            should_try_write: true
        }
    ));
    assert!(state.needs_write());
}

#[test]
fn silent_zmodem_abort_clears_pending_session_without_ui_event() {
    let mut receiver = zmodem2::Receiver::new().unwrap();
    let receiver_init = match receiver.poll() {
        Action::WriteWire(bytes) => bytes.to_vec(),
        other => panic!("receiver should advertise ZRINIT first, got {other:?}"),
    };

    let (events_tx, events_rx) = async_channel::unbounded();
    let listener = ChannelEventListener::builder_for_test()
        .with_terminal_events_tx(events_tx)
        .build();
    let (tx, rx) = mio_channel::channel();
    tx.send(Message::AbortZmodemSilently).unwrap();
    let mut loop_ = EventLoop::new(
        Arc::new(FairMutex::new(TerminalModel::mock(None, None))),
        listener,
        CapturingPty::default(),
        rx,
    );
    let mut state = State {
        zmodem: ZmodemState::Pending(PendingZmodemSession::new(
            ZmodemDirection::Upload,
            &receiver_init,
        )),
        ..State::default()
    };

    let result = loop_.drain_recv_channel(&mut state);
    let mut can_write = matches!(
        result,
        ChannelResult::Continue {
            should_try_write: true
        }
    );
    assert!(can_write);
    loop_.pty_write(&mut state, &mut can_write).unwrap();

    assert!(matches!(state.zmodem, ZmodemState::Inactive));
    assert_eq!(
        loop_.pty.writer.bytes,
        crate::terminal::zmodem::ZMODEM_ABORT_SEQUENCE
    );
    assert!(events_rx.is_empty());
}

#[test]
fn zmodem_cancel_discards_queued_transfer_bytes_before_abort() {
    let (events_tx, events_rx) = async_channel::unbounded();
    let listener = ChannelEventListener::builder_for_test()
        .with_terminal_events_tx(events_tx)
        .build();
    let (_tx, rx) = mio_channel::channel();
    let mut loop_ = EventLoop::new(
        Arc::new(FairMutex::new(TerminalModel::mock(None, None))),
        listener,
        CapturingPty::default(),
        rx,
    );
    let mut state = State {
        zmodem: ZmodemState::Active(ZmodemSession::Mock(MockZmodemSession {
            direction: ZmodemDirection::Upload,
            input: Vec::new(),
            events: Vec::new(),
            done: false,
        })),
        ..State::default()
    };
    state.write_list.push_back(super::PendingWrite {
        source: super::WriteSource::Zmodem,
        bytes: std::borrow::Cow::Borrowed(b"queued file data"),
    });
    state.writing = Some(super::Writing::new(super::PendingWrite {
        source: super::WriteSource::Zmodem,
        bytes: std::borrow::Cow::Borrowed(b"currently writing file data"),
    }));

    loop_.handle_zmodem_paths(
        &mut state,
        ZmodemTransferPaths::cancel(ZmodemDirection::Upload),
    );
    let mut can_write = true;
    loop_.pty_write(&mut state, &mut can_write).unwrap();

    assert!(matches!(state.zmodem, ZmodemState::Inactive));
    assert_eq!(loop_.pty.writer.bytes, ZMODEM_ABORT_SEQUENCE);
    assert!(matches!(
        events_rx.try_recv().unwrap(),
        TerminalEvent::Zmodem(ZmodemEvent::Cancelled {
            direction: ZmodemDirection::Upload
        })
    ));
}

#[test]
fn zmodem_paths_message_flushes_queued_wire_to_pty_writer() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("upload.txt");
    std::fs::write(&source_path, b"hello").unwrap();
    let mut receiver = zmodem2::Receiver::new().unwrap();
    let receiver_init = match receiver.poll() {
        Action::WriteWire(bytes) => bytes.to_vec(),
        other => panic!("receiver should advertise ZRINIT first, got {other:?}"),
    };

    let (tx, rx) = mio_channel::channel();
    tx.send(Message::ZmodemTransferPaths(ZmodemTransferPaths::upload(
        vec![source_path],
    )))
    .unwrap();
    let mut loop_ = EventLoop::new(
        Arc::new(FairMutex::new(TerminalModel::mock(None, None))),
        ChannelEventListener::new_for_test(),
        CapturingPty::default(),
        rx,
    );
    let mut state = State {
        zmodem: ZmodemState::Pending(PendingZmodemSession::new(
            ZmodemDirection::Upload,
            &receiver_init,
        )),
        ..State::default()
    };

    let result = loop_.drain_recv_channel(&mut state);
    let mut can_write = matches!(
        result,
        ChannelResult::Continue {
            should_try_write: true
        }
    );

    assert!(can_write);
    loop_.pty_write(&mut state, &mut can_write).unwrap();

    let written = &loop_.pty.writer.bytes;
    assert!(!written.is_empty());
    assert_starts_with_zfile_header(
        written,
        "channel-triggered ZMODEM upload should write ZFILE to PTY",
    );
    assert!(!state.needs_write());
}

#[test]
fn zmodem_upload_paths_without_pending_session_emit_failure() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("upload.txt");
    std::fs::write(&source_path, b"hello").unwrap();

    let (events_tx, events_rx) = async_channel::unbounded();
    let listener = ChannelEventListener::builder_for_test()
        .with_terminal_events_tx(events_tx)
        .build();
    let (_tx, rx) = mio_channel::channel();
    let mut loop_ = EventLoop::new(
        Arc::new(FairMutex::new(TerminalModel::mock(None, None))),
        listener,
        MockPty,
        rx,
    );
    let mut state = State::default();

    loop_.handle_zmodem_paths(&mut state, ZmodemTransferPaths::upload(vec![source_path]));

    assert!(matches!(state.zmodem, ZmodemState::Inactive));
    assert!(!state.needs_write());
    assert!(matches!(
        events_rx.try_recv().unwrap(),
        TerminalEvent::Zmodem(ZmodemEvent::Failed {
            direction: Some(ZmodemDirection::Upload),
            message,
        }) if message == "ZMODEM session is no longer active"
    ));
}

#[test]
fn zmodem_upload_paths_queue_zfile_after_receiver_init() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("upload.txt");
    std::fs::write(&source_path, b"hello").unwrap();

    let mut receiver = zmodem2::Receiver::new().unwrap();
    let receiver_init = match receiver.poll() {
        Action::WriteWire(bytes) => {
            let bytes = bytes.to_vec();
            receiver.wire_written(bytes.len());
            bytes
        }
        other => panic!("receiver should advertise ZRINIT first, got {other:?}"),
    };
    let (events_tx, events_rx) = async_channel::unbounded();
    let listener = ChannelEventListener::builder_for_test()
        .with_terminal_events_tx(events_tx)
        .build();
    let (_tx, rx) = mio_channel::channel();
    let mut loop_ = EventLoop::new(
        Arc::new(FairMutex::new(TerminalModel::mock(None, None))),
        listener,
        MockPty,
        rx,
    );
    let mut state = State {
        zmodem: ZmodemState::Pending(PendingZmodemSession::new(
            ZmodemDirection::Upload,
            &receiver_init,
        )),
        ..State::default()
    };

    loop_.handle_zmodem_paths(&mut state, ZmodemTransferPaths::upload(vec![source_path]));

    let queued_wire = state
        .write_list
        .iter()
        .flat_map(|pending| pending.bytes.iter().copied())
        .collect::<Vec<_>>();
    assert_starts_with_zfile_header(
        &queued_wire,
        "selecting a file for an active rz session should queue ZFILE",
    );
    assert_zcrcw_is_followed_by_xon(&queued_wire);
    assert!(matches!(state.zmodem, ZmodemState::Active(_)));
    assert!(matches!(
        events_rx.try_recv().unwrap(),
        TerminalEvent::Zmodem(ZmodemEvent::Started {
            direction: ZmodemDirection::Upload
        })
    ));
    assert!(events_rx.is_empty());

    let mut receiver_input = queued_wire;
    let mut receiver_to_upload = Vec::new();
    loop {
        if !receiver_input.is_empty() {
            submit_receiver_wire(&mut receiver, &mut receiver_input);
        }

        match receiver.poll() {
            Action::WriteWire(bytes) => {
                let bytes = bytes.to_vec();
                let len = bytes.len();
                receiver_to_upload.extend(bytes);
                receiver.wire_written(len);
            }
            Action::Event(zmodem2::Event::FileStarted(_)) => {}
            Action::Idle if receiver_input.is_empty() => break,
            Action::Idle => {}
            other => panic!("unexpected receiver action after ZFILE: {other:?}"),
        }
    }
    assert!(receiver_to_upload.starts_with(b"**\x18B09"));

    let output = EventLoop::<MockPty>::process_pty_bytes(
        &loop_.event_listener,
        &mut state,
        &receiver_to_upload,
    );

    assert_eq!(output, Vec::<u8>::new());
    assert!(matches!(
        events_rx.try_recv().unwrap(),
        TerminalEvent::Zmodem(ZmodemEvent::FileStarted {
            direction: ZmodemDirection::Upload,
            name,
            size: Some(5),
            ..
        }) if name == "upload.txt"
    ));
    assert!(matches!(
        events_rx.try_recv().unwrap(),
        TerminalEvent::Zmodem(ZmodemEvent::Progress {
            direction: ZmodemDirection::Upload,
            name,
            transferred: 5,
            total: Some(5),
        }) if name == "upload.txt"
    ));
}

#[test]
fn zmodem_detected_rz_then_selected_file_reaches_upload_progress() {
    let _flag = warp_core::features::FeatureFlag::Lrzsz.override_enabled(true);
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("upload.txt");
    std::fs::write(&source_path, b"hello").unwrap();

    let mut receiver = zmodem2::Receiver::new().unwrap();
    let receiver_init = match receiver.poll() {
        Action::WriteWire(bytes) => {
            let bytes = bytes.to_vec();
            receiver.wire_written(bytes.len());
            bytes
        }
        other => panic!("receiver should advertise ZRINIT first, got {other:?}"),
    };
    let (events_tx, events_rx) = async_channel::unbounded();
    let listener = ChannelEventListener::builder_for_test()
        .with_terminal_events_tx(events_tx)
        .build();
    let (_tx, rx) = mio_channel::channel();
    let mut loop_ = EventLoop::new(
        Arc::new(FairMutex::new(TerminalModel::mock(None, None))),
        listener,
        MockPty,
        rx,
    );
    let mut state = State::default();

    let ordinary_output =
        EventLoop::<MockPty>::process_pty_bytes(&loop_.event_listener, &mut state, &receiver_init);

    assert_eq!(ordinary_output, Vec::<u8>::new());
    assert!(matches!(state.zmodem, ZmodemState::Pending(_)));
    assert!(matches!(
        events_rx.try_recv().unwrap(),
        TerminalEvent::Zmodem(ZmodemEvent::UploadRequested)
    ));

    loop_.handle_zmodem_paths(&mut state, ZmodemTransferPaths::upload(vec![source_path]));

    let queued_wire = state
        .write_list
        .iter()
        .flat_map(|pending| pending.bytes.iter().copied())
        .collect::<Vec<_>>();
    assert!(!queued_wire.is_empty());

    let mut receiver_input = queued_wire;
    let mut receiver_to_upload = Vec::new();
    loop {
        if !receiver_input.is_empty() {
            submit_receiver_wire(&mut receiver, &mut receiver_input);
        }

        match receiver.poll() {
            Action::WriteWire(bytes) => {
                let bytes = bytes.to_vec();
                receiver.wire_written(bytes.len());
                receiver_to_upload.extend(bytes);
            }
            Action::Event(zmodem2::Event::FileStarted(_)) => {}
            Action::Idle if receiver_input.is_empty() => break,
            Action::Idle => {}
            other => panic!("unexpected receiver action after ZFILE: {other:?}"),
        }
    }
    assert!(receiver_to_upload.starts_with(b"**\x18B09"));

    let output = EventLoop::<MockPty>::process_pty_bytes(
        &loop_.event_listener,
        &mut state,
        &receiver_to_upload,
    );

    assert_eq!(output, Vec::<u8>::new());
    assert!(matches!(
        events_rx.try_recv().unwrap(),
        TerminalEvent::Zmodem(ZmodemEvent::Started {
            direction: ZmodemDirection::Upload
        })
    ));
    assert!(matches!(
        events_rx.try_recv().unwrap(),
        TerminalEvent::Zmodem(ZmodemEvent::FileStarted {
            direction: ZmodemDirection::Upload,
            name,
            size: Some(5),
            ..
        }) if name == "upload.txt"
    ));
    assert!(matches!(
        events_rx.try_recv().unwrap(),
        TerminalEvent::Zmodem(ZmodemEvent::Progress {
            direction: ZmodemDirection::Upload,
            name,
            transferred: 5,
            total: Some(5),
        }) if name == "upload.txt"
    ));
}

#[test]
fn zmodem_detected_rz_split_init_then_selected_file_waits_for_rest_and_reaches_progress() {
    let _flag = warp_core::features::FeatureFlag::Lrzsz.override_enabled(true);
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("upload.txt");
    std::fs::write(&source_path, b"hello").unwrap();

    let mut receiver = zmodem2::Receiver::new().unwrap();
    let receiver_init = match receiver.poll() {
        Action::WriteWire(bytes) => {
            let bytes = bytes.to_vec();
            receiver.wire_written(bytes.len());
            bytes
        }
        other => panic!("receiver should advertise ZRINIT first, got {other:?}"),
    };
    assert!(receiver_init.starts_with(b"**\x18B01"));
    let (events_tx, events_rx) = async_channel::unbounded();
    let listener = ChannelEventListener::builder_for_test()
        .with_terminal_events_tx(events_tx)
        .build();
    let (_tx, rx) = mio_channel::channel();
    let mut loop_ = EventLoop::new(
        Arc::new(FairMutex::new(TerminalModel::mock(None, None))),
        listener,
        MockPty,
        rx,
    );
    let mut state = State::default();

    let ordinary_output = EventLoop::<MockPty>::process_pty_bytes(
        &loop_.event_listener,
        &mut state,
        &receiver_init[..6],
    );

    assert_eq!(ordinary_output, Vec::<u8>::new());
    assert!(matches!(state.zmodem, ZmodemState::Inactive));
    assert!(events_rx.is_empty());

    let ordinary_output = EventLoop::<MockPty>::process_pty_bytes(
        &loop_.event_listener,
        &mut state,
        &receiver_init[6..],
    );

    assert_eq!(ordinary_output, Vec::<u8>::new());
    assert!(matches!(state.zmodem, ZmodemState::Pending(_)));
    assert!(matches!(
        events_rx.try_recv().unwrap(),
        TerminalEvent::Zmodem(ZmodemEvent::UploadRequested)
    ));

    loop_.handle_zmodem_paths(&mut state, ZmodemTransferPaths::upload(vec![source_path]));
    assert!(matches!(state.zmodem, ZmodemState::Active(_)));
    assert!(matches!(
        events_rx.try_recv().unwrap(),
        TerminalEvent::Zmodem(ZmodemEvent::Started {
            direction: ZmodemDirection::Upload
        })
    ));
    let queued_wire = state
        .write_list
        .iter()
        .flat_map(|pending| pending.bytes.iter().copied())
        .collect::<Vec<_>>();
    assert_starts_with_zfile_header(&queued_wire, "rest of receiver init should queue ZFILE");
    assert_zcrcw_is_followed_by_xon(&queued_wire);
}

fn assert_zcrcw_is_followed_by_xon(bytes: &[u8]) {
    bytes
        .windows(2)
        .position(|window| window == b"\x18k")
        .expect("ZMODEM wire should contain a ZCRCW subpacket terminator");
    assert_eq!(
        bytes.last(),
        Some(&0x11),
        "ZCRCW subpacket should be followed by CRC bytes and a trailing XON for lrzsz-compatible PTY flow control"
    );
}

fn assert_starts_with_zfile_header(bytes: &[u8], context: &str) {
    assert!(
        bytes.starts_with(b"*\x18C\x04") || bytes.starts_with(b"**\x18B04"),
        "{context}, got {bytes:?}"
    );
}

fn submit_receiver_wire(receiver: &mut zmodem2::Receiver, receiver_input: &mut Vec<u8>) {
    if receiver_input == b"\x11" {
        receiver_input.clear();
        return;
    }

    let consumed = receiver.submit_wire(receiver_input).unwrap();
    assert!(consumed > 0, "receiver did not consume {receiver_input:?}");
    receiver_input.drain(..consumed);
    if receiver_input == b"\x11" {
        receiver_input.clear();
    }
}

struct MockPty;

impl local_tty::EventedReadWrite for MockPty {
    type Reader = std::io::Empty;
    type Writer = std::io::Sink;

    fn register(&mut self, _: &mio::Poll, _: mio::Interest) -> std::io::Result<()> {
        Ok(())
    }

    fn reregister(&mut self, _: &mio::Poll, _: mio::Interest) -> std::io::Result<()> {
        Ok(())
    }

    fn deregister(&mut self, _: &mio::Poll) -> std::io::Result<()> {
        Ok(())
    }

    fn reader(&mut self) -> &mut Self::Reader {
        unimplemented!()
    }

    fn read_token(&self) -> mio::Token {
        mio::Token(0)
    }

    fn writer(&mut self) -> &mut Self::Writer {
        unimplemented!()
    }

    fn write_token(&self) -> mio::Token {
        mio::Token(0)
    }
}

#[derive(Default)]
struct CapturingPty {
    reader: std::io::Empty,
    writer: CapturingWriter,
}

#[derive(Default)]
struct CapturingWriter {
    bytes: Vec<u8>,
}

impl std::io::Write for CapturingWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.bytes.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl local_tty::EventedReadWrite for CapturingPty {
    type Reader = std::io::Empty;
    type Writer = CapturingWriter;

    fn register(&mut self, _: &mio::Poll, _: mio::Interest) -> std::io::Result<()> {
        Ok(())
    }

    fn reregister(&mut self, _: &mio::Poll, _: mio::Interest) -> std::io::Result<()> {
        Ok(())
    }

    fn deregister(&mut self, _: &mio::Poll) -> std::io::Result<()> {
        Ok(())
    }

    fn reader(&mut self) -> &mut Self::Reader {
        &mut self.reader
    }

    fn read_token(&self) -> mio::Token {
        mio::Token(0)
    }

    fn writer(&mut self) -> &mut Self::Writer {
        &mut self.writer
    }

    fn write_token(&self) -> mio::Token {
        mio::Token(0)
    }
}

impl local_tty::EventedPty for CapturingPty {
    fn child_event_token(&self) -> mio::Token {
        mio::Token(0)
    }

    fn next_child_event(&mut self) -> Option<local_tty::ChildEvent> {
        None
    }

    fn on_resize(&mut self, _: &crate::terminal::SizeInfo) {}

    fn kill(self) -> anyhow::Result<()> {
        Ok(())
    }
}

impl local_tty::EventedPty for MockPty {
    fn child_event_token(&self) -> mio::Token {
        mio::Token(0)
    }

    fn next_child_event(&mut self) -> Option<local_tty::ChildEvent> {
        None
    }

    fn on_resize(&mut self, _: &crate::terminal::SizeInfo) {}

    fn kill(self) -> anyhow::Result<()> {
        Ok(())
    }
}
