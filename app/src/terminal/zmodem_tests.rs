use std::{fs::File, io::Write};
use tempfile::tempdir;
use zmodem2::{Action, Event, FileInfo, Position};

use super::{
    detect_zmodem_session, prepare_zmodem_receiver_wire_bytes, prepare_zmodem_wire_bytes,
    sanitize_wire_file_name, unused_path, CompletedDownloadFile, DownloadFile, DownloadSession,
    PendingZmodemSession, UploadSession, ZmodemDetection, ZmodemDetector, ZmodemDetectorResult,
    ZmodemDirection, ZmodemEvent, ZmodemSession, ZmodemTransferPaths, ZMODEM_ABORT_SEQUENCE,
    ZMODEM_FAST_WINDOW_SIZE, ZMODEM_ZRINIT_CANFC32, ZMODEM_ZRINIT_CANOVIO,
};

#[test]
fn detects_download_header_after_ordinary_output() {
    let bytes = b"ready\r\n**\x18B00000000000000";

    assert_eq!(
        detect_zmodem_session(bytes),
        Some(ZmodemDetection {
            direction: ZmodemDirection::Download,
            start_index: 7,
        })
    );
}

#[test]
fn detects_upload_header_after_ordinary_output() {
    let bytes = b"ready\r\n**\x18B01000000000000";

    assert_eq!(
        detect_zmodem_session(bytes),
        Some(ZmodemDetection {
            direction: ZmodemDirection::Upload,
            start_index: 7,
        })
    );
}

#[test]
fn ignores_non_zmodem_header() {
    assert_eq!(
        detect_zmodem_session(b"ready\r\n**\x18B99000000000000"),
        None
    );
    assert_eq!(
        detect_zmodem_session(b"ready\r\n**\x18B01not-hex-data"),
        None
    );
    assert_eq!(detect_zmodem_session(b"ready\r\n"), None);
}

#[test]
fn skips_invalid_header_before_valid_header() {
    let bytes = b"bad**\x18B99000000000000good**\x18B00000000000000";

    assert_eq!(
        detect_zmodem_session(bytes),
        Some(ZmodemDetection {
            direction: ZmodemDirection::Download,
            start_index: 25,
        })
    );
}

#[test]
fn detector_keeps_split_header_until_frame_arrives() {
    let mut detector = ZmodemDetector::default();

    assert_eq!(
        detector.push(b"ordinary**\x18"),
        ZmodemDetectorResult::Ordinary(b"ordinary".to_vec())
    );
    assert_eq!(
        detector.push(b"B01000000000000payload"),
        ZmodemDetectorResult::Detected {
            detection: ZmodemDetection {
                direction: ZmodemDirection::Upload,
                start_index: 0,
            },
            ordinary_output: Vec::new(),
            zmodem_input: b"**\x18B01000000000000payload".to_vec(),
        }
    );
}

#[test]
fn detector_waits_for_complete_hex_header_before_detecting() {
    let mut detector = ZmodemDetector::default();

    assert_eq!(
        detector.push(b"ordinary**\x18B01"),
        ZmodemDetectorResult::Ordinary(b"ordinary".to_vec())
    );
    assert_eq!(
        detector.push(b"000000000000"),
        ZmodemDetectorResult::Detected {
            detection: ZmodemDetection {
                direction: ZmodemDirection::Upload,
                start_index: 0,
            },
            ordinary_output: Vec::new(),
            zmodem_input: b"**\x18B01000000000000".to_vec(),
        }
    );
}

#[test]
fn detector_flushes_non_zmodem_partial_header() {
    let mut detector = ZmodemDetector::default();

    assert_eq!(
        detector.push(b"ordinary**\x18"),
        ZmodemDetectorResult::Ordinary(b"ordinary".to_vec())
    );
    assert_eq!(
        detector.push(b"not-zmodem"),
        ZmodemDetectorResult::Ordinary(b"**\x18not-zmodem".to_vec())
    );
}

#[test]
fn sanitize_wire_file_name_uses_basename() {
    assert_eq!(
        sanitize_wire_file_name(b"../../server/path/file.txt").unwrap(),
        "file.txt"
    );
    assert_eq!(
        sanitize_wire_file_name(br"C:\Users\name\file.txt").unwrap(),
        "file.txt"
    );
}

#[test]
fn sanitize_wire_file_name_rejects_empty_or_parent_names() {
    assert!(sanitize_wire_file_name(b"").is_err());
    assert!(sanitize_wire_file_name(b".").is_err());
    assert!(sanitize_wire_file_name(b"..").is_err());
    assert!(sanitize_wire_file_name(b"../../..").is_err());
}

#[test]
fn unused_path_adds_suffix_when_file_exists() {
    let dir = tempdir().unwrap();
    let first_path = dir.path().join("artifact.txt");
    let second_path = dir.path().join("artifact (1).txt");
    File::create(&first_path).unwrap();
    File::create(&second_path).unwrap();

    assert_eq!(
        unused_path(dir.path(), "artifact.txt"),
        dir.path().join("artifact (2).txt")
    );
}

#[test]
fn unused_path_preserves_extensionless_names() {
    let dir = tempdir().unwrap();
    let first_path = dir.path().join("artifact");
    File::create(&first_path).unwrap();

    assert_eq!(
        unused_path(dir.path(), "artifact"),
        dir.path().join("artifact (1)")
    );
}

#[test]
fn unused_path_handles_dotfiles() {
    let dir = tempdir().unwrap();
    let first_path = dir.path().join(".env");
    File::create(&first_path).unwrap();

    assert_eq!(unused_path(dir.path(), ".env"), dir.path().join(".env (1)"));
}

#[test]
fn upload_session_sends_file_to_receiver() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("upload.txt");
    std::fs::write(&source_path, b"hello from upload").unwrap();

    let mut upload = UploadSession::new(vec![source_path.clone()]).unwrap();
    let mut receiver = zmodem2::Receiver::new().unwrap();
    let mut upload_input = Vec::new();
    let mut receiver_input = Vec::new();
    let mut received = Vec::new();
    let mut upload_events = Vec::new();
    let mut receiver_done = false;

    for _ in 0..10_000 {
        let upload_done = upload
            .drain_actions(&mut |bytes| receiver_input.extend(bytes), &mut |event| {
                upload_events.push(event)
            })
            .unwrap();

        match receiver.poll() {
            Action::WriteWire(bytes) => {
                let bytes = bytes.to_vec();
                let len = bytes.len();
                upload_input.extend(bytes);
                receiver.wire_written(len);
            }
            Action::WriteFile(bytes) => {
                let bytes = bytes.to_vec();
                let len = bytes.len();
                received.extend(bytes);
                receiver.file_written(len).unwrap();
            }
            Action::Event(event) => match event {
                Event::FileStarted(info) => {
                    assert_eq!(info.name, b"upload.txt");
                    assert_eq!(info.size, Some(Position::new(17)));
                }
                Event::FileCompleted => {}
                Event::SessionCompleted => receiver_done = true,
                Event::Aborted => panic!("receiver aborted"),
                _ => {}
            },
            Action::ReadFile { .. } => panic!("receiver should not read local files"),
            Action::Idle => {
                if !receiver_input.is_empty() {
                    submit_receiver_wire(&mut receiver, &mut receiver_input);
                }
            }
            _ => {}
        }

        if !upload_input.is_empty() {
            upload.input.extend(upload_input.drain(..));
            upload_input.clear();
        }

        if upload_done && receiver_done {
            break;
        }
    }

    assert_eq!(received, b"hello from upload");
    assert!(upload_events.iter().any(|event| {
        matches!(
            event,
            ZmodemEvent::Completed {
                direction: ZmodemDirection::Upload
            }
        )
    }));
}

#[test]
fn upload_session_does_not_start_file_until_receiver_accepts_zfile() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("upload.txt");
    std::fs::write(&source_path, b"hello").unwrap();

    let mut upload = UploadSession::new(vec![source_path]).unwrap();
    let mut events = Vec::new();
    let mut wire = Vec::new();

    upload
        .drain_actions(&mut |bytes| wire.extend(bytes), &mut |event| {
            events.push(event)
        })
        .unwrap();

    assert!(!wire.is_empty());
    assert_eq!(
        events,
        vec![ZmodemEvent::Started {
            direction: ZmodemDirection::Upload,
        }]
    );
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

#[test]
fn upload_session_appends_xon_after_zcrcw_zfile_wire() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("upload.txt");
    std::fs::write(&source_path, b"hello").unwrap();

    let mut upload = UploadSession::new(vec![source_path]).unwrap();
    let mut receiver = zmodem2::Receiver::new().unwrap();
    let mut upload_input = Vec::new();
    match receiver.poll() {
        Action::WriteWire(bytes) => {
            let len = bytes.len();
            upload_input.extend_from_slice(bytes);
            receiver.wire_written(len);
        }
        other => panic!("receiver should advertise ZRINIT first, got {other:?}"),
    }
    upload.input.extend(upload_input);

    let mut upload_to_receiver = Vec::new();
    upload
        .drain_actions(&mut |bytes| upload_to_receiver.extend(bytes), &mut |_| {})
        .unwrap();

    assert_zcrcw_is_followed_by_xon(&upload_to_receiver);

    let mut receiver_input = upload_to_receiver;
    loop {
        if !receiver_input.is_empty() {
            submit_receiver_wire(&mut receiver, &mut receiver_input);
        }

        match receiver.poll() {
            Action::WriteWire(bytes) => {
                let len = bytes.len();
                receiver.wire_written(len);
            }
            Action::Event(Event::FileStarted(info)) => {
                assert_eq!(info.name, b"upload.txt");
                assert_eq!(info.size, Some(Position::new(5)));
                break;
            }
            Action::Idle if receiver_input.is_empty() => {
                panic!("receiver did not parse ZFILE wire")
            }
            Action::Idle => {}
            other => panic!("unexpected receiver action after ZFILE: {other:?}"),
        }
    }
}

#[test]
fn upload_session_preserves_zmodem2_zfile_wire_bytes() {
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

    let expected_wire = raw_zmodem2_zfile_wire(b"upload.txt", 5, &receiver_init);
    let mut upload = UploadSession::new_after_receiver_init(vec![source_path]).unwrap();
    upload.input.extend(receiver_init);

    let mut queued_wire = Vec::new();
    upload
        .drain_actions(&mut |bytes| queued_wire.extend(bytes), &mut |_| {})
        .unwrap();

    assert_eq!(
        &queued_wire[..expected_wire.len()],
        expected_wire.as_slice()
    );
    assert_eq!(
        &queued_wire[expected_wire.len()..],
        b"\x11",
        "the PTY wire may only add the lrzsz-compatible XON after ZCRCW"
    );
}

fn raw_zmodem2_zfile_wire(file_name: &[u8], file_size: u32, receiver_init: &[u8]) -> Vec<u8> {
    let mut sender = zmodem2::Sender::new().unwrap();
    while let Action::WriteWire(bytes) = sender.poll() {
        let len = bytes.len();
        sender.wire_written(len);
    }
    sender
        .start_file(FileInfo::new(file_name, Some(Position::new(file_size))))
        .unwrap();
    assert!(sender.submit_wire(receiver_init).unwrap() > 0);

    let mut wire = Vec::new();
    while let Action::WriteWire(bytes) = sender.poll() {
        wire.extend_from_slice(bytes);
        let len = bytes.len();
        sender.wire_written(len);
    }
    wire
}

#[test]
fn prepare_wire_preserves_protocol_bytes_and_appends_xon_after_zcrcw() {
    assert_eq!(prepare_zmodem_wire_bytes(b"abc"), b"abc");
    assert_eq!(
        prepare_zmodem_wire_bytes(b"*\x18C\x04upload.txt\x18h"),
        b"*\x18C\x04upload.txt\x18h"
    );
    assert_eq!(
        prepare_zmodem_wire_bytes(b"payload\x18k"),
        b"payload\x18k\x11"
    );
}

#[test]
fn receiver_zrinit_wire_advertises_fast_window_for_downloads() {
    let mut receiver = zmodem2::Receiver::new().unwrap();
    let receiver_init = match receiver.poll() {
        Action::WriteWire(bytes) => {
            let bytes = bytes.to_vec();
            receiver.wire_written(bytes.len());
            bytes
        }
        other => panic!("receiver should advertise ZRINIT first, got {other:?}"),
    };

    let rewritten = prepare_zmodem_receiver_wire_bytes(&receiver_init);

    assert_fast_zrinit(&rewritten);
    let mut sender = zmodem2::Sender::new().unwrap();
    while let Action::WriteWire(bytes) = sender.poll() {
        let len = bytes.len();
        sender.wire_written(len);
    }
    assert!(
        sender.submit_wire(&rewritten).unwrap() > 0,
        "rewritten ZRINIT must keep a valid CRC"
    );
}

#[test]
fn upload_session_normalizes_remote_zrinit_for_larger_windows() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("upload.txt");
    std::fs::write(&source_path, b"hello").unwrap();
    let lrzsz_zrinit = b"**\x18B0100000023be50\r\x8a\x11";
    let pending = PendingZmodemSession::new(ZmodemDirection::Upload, lrzsz_zrinit);
    let mut upload = match pending
        .start(ZmodemTransferPaths::upload(vec![source_path]))
        .unwrap()
    {
        ZmodemSession::Upload(session) => session,
        ZmodemSession::Download(_) => panic!("pending upload should create upload session"),
        #[cfg(test)]
        ZmodemSession::Mock(_) => panic!("pending upload should not create mock session"),
    };

    let mut wire = Vec::new();
    upload
        .drain_actions(&mut |bytes| wire.extend(bytes), &mut |_| {})
        .unwrap();

    assert!(
        wire.starts_with(b"*\x18C\x04"),
        "normalized lrzsz ZRINIT should drive sender directly to ZFILE"
    );
}

#[test]
fn upload_session_normalizes_split_remote_zrinit() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("upload.txt");
    std::fs::write(&source_path, b"hello").unwrap();
    let lrzsz_zrinit = b"**\x18B0100000023be50\r\x8a\x11";
    let mut upload = UploadSession::new_after_receiver_init(vec![source_path]).unwrap();
    upload.input.extend(&lrzsz_zrinit[..6]);
    let mut session = ZmodemSession::Upload(upload);
    session.append_input(&lrzsz_zrinit[6..]);
    let mut upload = match session {
        ZmodemSession::Upload(session) => session,
        ZmodemSession::Download(_) => unreachable!(),
        #[cfg(test)]
        ZmodemSession::Mock(_) => unreachable!(),
    };

    let mut wire = Vec::new();
    upload
        .drain_actions(&mut |bytes| wire.extend(bytes), &mut |_| {})
        .unwrap();

    assert!(
        wire.starts_with(b"*\x18C\x04"),
        "split lrzsz ZRINIT should still drive sender directly to ZFILE"
    );
}

fn assert_fast_zrinit(bytes: &[u8]) {
    assert!(bytes.starts_with(b"**\x18B01"));
    let payload = &bytes[4..18];
    let header = String::from_utf8(payload.to_vec()).unwrap();
    let flags = (0..7)
        .map(|index| u8::from_str_radix(&header[index * 2..index * 2 + 2], 16).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        u16::from_le_bytes([flags[1], flags[2]]),
        ZMODEM_FAST_WINDOW_SIZE
    );
    assert_eq!(
        flags[4] & (ZMODEM_ZRINIT_CANOVIO | ZMODEM_ZRINIT_CANFC32),
        ZMODEM_ZRINIT_CANOVIO | ZMODEM_ZRINIT_CANFC32
    );
}

#[test]
fn pending_upload_consumes_detected_receiver_init_before_sending_file_data() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("upload.txt");
    std::fs::write(&source_path, b"hello from upload").unwrap();

    let mut receiver = zmodem2::Receiver::new().unwrap();
    let mut receiver_init = Vec::new();
    match receiver.poll() {
        Action::WriteWire(bytes) => {
            let bytes = bytes.to_vec();
            let len = bytes.len();
            receiver_init.extend(bytes);
            receiver.wire_written(len);
        }
        other => panic!("receiver should advertise ZRINIT first, got {other:?}"),
    }
    assert!(receiver_init.starts_with(b"**\x18B01"));

    let pending = PendingZmodemSession::new(ZmodemDirection::Upload, &receiver_init[..6]);
    let mut upload = match pending
        .start(ZmodemTransferPaths::upload(vec![source_path.clone()]))
        .unwrap()
    {
        ZmodemSession::Upload(session) => session,
        ZmodemSession::Download(_) => panic!("pending upload should create upload session"),
        #[cfg(test)]
        ZmodemSession::Mock(_) => panic!("pending upload should not create mock session"),
    };
    upload.input.extend(receiver_init[6..].iter().copied());

    let mut upload_to_receiver = Vec::new();
    let mut upload_events = Vec::new();
    upload
        .drain_actions(
            &mut |bytes| upload_to_receiver.extend(bytes),
            &mut |event| upload_events.push(event),
        )
        .unwrap();

    assert!(
        !upload_to_receiver.starts_with(b"**\x18B00"),
        "pending rz upload must not restart the sender-side ZRQINIT handshake"
    );
    assert_starts_with_zfile_header(
        &upload_to_receiver,
        "receiver init should drive the sender directly to ZFILE",
    );

    let mut receiver_input = upload_to_receiver;
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
            Action::Event(Event::FileStarted(info)) => {
                assert_eq!(info.name, b"upload.txt");
                assert_eq!(info.size, Some(Position::new(17)));
            }
            Action::Idle if receiver_input.is_empty() => break,
            Action::Idle => {}
            other => panic!("unexpected receiver action after ZFILE: {other:?}"),
        }
    }
    assert!(receiver_to_upload.starts_with(b"**\x18B09"));

    upload.input.extend(receiver_to_upload);
    let mut progress_events = Vec::new();
    upload
        .drain_actions(&mut |_| {}, &mut |event| progress_events.push(event))
        .unwrap();

    assert!(progress_events.iter().any(|event| {
        matches!(
            event,
            ZmodemEvent::FileStarted {
                direction: ZmodemDirection::Upload,
                name,
                size: Some(17),
                ..
            } if name == "upload.txt"
        )
    }));
    assert!(progress_events.iter().any(|event| {
        matches!(
            event,
            ZmodemEvent::Progress {
                direction: ZmodemDirection::Upload,
                name,
                transferred,
                total: Some(17),
            } if name == "upload.txt" && *transferred > 0
        )
    }));
}

#[test]
fn download_session_receives_file_from_sender() {
    let dir = tempdir().unwrap();
    let received_path = dir.path().join("remote.txt");
    let mut download = DownloadSession::new(dir.path().to_path_buf()).unwrap();
    let mut sender = zmodem2::Sender::new().unwrap();
    sender
        .start_file(FileInfo::new(
            b"remote.txt",
            Some(Position::new("downloaded data".len() as u32)),
        ))
        .unwrap();

    let mut download_input = Vec::new();
    let mut sender_input = Vec::new();
    let mut download_events = Vec::new();
    let mut file_completed_after_persist = false;
    let mut sender_done = false;
    let payload = b"downloaded data";

    for _ in 0..10_000 {
        let download_done = download
            .drain_actions(&mut |bytes| sender_input.extend(bytes), &mut |event| {
                if matches!(
                    &event,
                    ZmodemEvent::FileCompleted {
                        direction: ZmodemDirection::Download,
                        path: Some(path),
                        ..
                    } if path == &received_path
                ) {
                    file_completed_after_persist = received_path.exists();
                }
                download_events.push(event)
            })
            .unwrap();

        match sender.poll() {
            Action::WriteWire(bytes) => {
                let bytes = bytes.to_vec();
                let len = bytes.len();
                download_input.extend(bytes);
                sender.wire_written(len);
            }
            Action::ReadFile { offset, max_len } => {
                let offset = offset.get() as usize;
                let end = (offset + max_len).min(payload.len());
                sender.submit_file(&payload[offset..end]).unwrap();
            }
            Action::Event(event) => match event {
                Event::FileCompleted => {
                    sender.finish().unwrap();
                }
                Event::SessionCompleted => sender_done = true,
                Event::FileStarted(_) => {}
                Event::Aborted => panic!("sender aborted"),
                _ => {}
            },
            Action::WriteFile(_) => panic!("sender should not write local files"),
            Action::Idle => {
                if !sender_input.is_empty() {
                    let consumed = sender.submit_wire(&sender_input).unwrap();
                    sender_input.drain(..consumed);
                }
            }
            _ => {}
        }

        if !download_input.is_empty() {
            download.input.extend(download_input.drain(..));
            download_input.clear();
        }

        if download_done && sender_done {
            break;
        }
    }

    assert_eq!(std::fs::read(received_path).unwrap(), payload);
    assert_eq!(zmodem_temp_paths(dir.path()).len(), 0);
    assert!(file_completed_after_persist);
    assert!(download_events.iter().any(|event| {
        matches!(
            event,
            ZmodemEvent::Completed {
                direction: ZmodemDirection::Download
            }
        )
    }));
}

#[test]
fn download_session_cancel_removes_partial_temp_file_only() {
    let dir = tempdir().unwrap();
    let final_path = dir.path().join("remote.txt");
    let mut temp_file = tempfile::Builder::new()
        .prefix(".warp-zmodem-")
        .suffix(".part")
        .tempfile_in(dir.path())
        .unwrap();
    let temp_path = temp_file.path().to_path_buf();
    temp_file.write_all(b"partial").unwrap();

    let mut download = DownloadSession::new(dir.path().to_path_buf()).unwrap();
    download.current_file = Some(DownloadFile {
        temp_file,
        name: String::from("remote.txt"),
        final_path: final_path.clone(),
        size: Some(1024),
        transferred: 7,
    });

    let cancel_bytes = ZmodemSession::Download(download).cancel();

    assert_eq!(cancel_bytes, ZMODEM_ABORT_SEQUENCE);
    assert!(!final_path.exists());
    assert!(!temp_path.exists());
}

#[test]
fn download_session_cancel_removes_completed_temp_file_before_commit() {
    let dir = tempdir().unwrap();
    let final_path = dir.path().join("remote.txt");
    let mut temp_file = tempfile::Builder::new()
        .prefix(".warp-zmodem-")
        .suffix(".part")
        .tempfile_in(dir.path())
        .unwrap();
    let temp_path = temp_file.path().to_path_buf();
    temp_file.write_all(b"complete but not committed").unwrap();

    let mut download = DownloadSession::new(dir.path().to_path_buf()).unwrap();
    download.completed_files.push(CompletedDownloadFile {
        temp_file,
        name: String::from("remote.txt"),
        final_path: final_path.clone(),
    });

    let cancel_bytes = ZmodemSession::Download(download).cancel();

    assert_eq!(cancel_bytes, ZMODEM_ABORT_SEQUENCE);
    assert!(!final_path.exists());
    assert!(!temp_path.exists());
}

#[test]
fn download_session_reserves_completed_final_names_before_commit() {
    let dir = tempdir().unwrap();
    let first_final_path = dir.path().join("remote.txt");
    let second_final_path = dir.path().join("remote (1).txt");
    let temp_file = tempfile::Builder::new()
        .prefix(".warp-zmodem-")
        .suffix(".part")
        .tempfile_in(dir.path())
        .unwrap();

    let mut download = DownloadSession::new(dir.path().to_path_buf()).unwrap();
    download.completed_files.push(CompletedDownloadFile {
        temp_file,
        name: String::from("remote.txt"),
        final_path: first_final_path,
    });

    assert_eq!(download.unused_final_path("remote.txt"), second_final_path);
}

fn zmodem_temp_paths(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(".warp-zmodem-") && name.ends_with(".part"))
        })
        .collect()
}
