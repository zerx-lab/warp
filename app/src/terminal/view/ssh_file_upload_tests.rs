use super::*;

use crate::test_util::terminal::{add_window_with_terminal, initialize_app_for_terminal_view};
use warp_core::command::ExitCode;
use warpui::{App, ViewHandle};

fn ssh_connection() -> InteractiveSshCommand {
    InteractiveSshCommand {
        host: Some(String::from("user@example.com")),
        port: None,
    }
}

#[test]
fn clear_completed_upload_removes_finished_upload() {
    App::test((), |mut app| async move {
        let upload = add_file_upload_view(&mut app);

        upload.update(&mut app, |file_upload, ctx| {
            let (upload_id, _) = file_upload.start_file_upload(
                "user@example.com",
                &[String::from("H:\\Downloads\\ddl.sql")],
                &Some(String::from("/tmp")),
                &ssh_connection(),
                ctx,
            );

            file_upload.file_upload_finished(upload_id, &ExitCode::from(0), ctx);
            assert!(file_upload.has_upload());

            file_upload.clear_completed_upload(upload_id, ctx);

            assert!(!file_upload.has_upload());
        });
    })
}

#[test]
fn clear_completed_upload_keeps_in_progress_upload() {
    App::test((), |mut app| async move {
        let upload = add_file_upload_view(&mut app);

        upload.update(&mut app, |file_upload, ctx| {
            let (upload_id, _) = file_upload.start_file_upload(
                "user@example.com",
                &[String::from("H:\\Downloads\\ddl.sql")],
                &Some(String::from("/tmp")),
                &ssh_connection(),
                ctx,
            );

            file_upload.clear_completed_upload(upload_id, ctx);

            assert!(file_upload.has_upload());
            assert!(file_upload.uploads.contains_key(&upload_id));
        });
    })
}

fn add_file_upload_view(app: &mut App) -> ViewHandle<FileUpload> {
    initialize_app_for_terminal_view(app);
    let terminal = add_window_with_terminal(app, None);
    terminal.read(app, |view, _| view.ssh_file_upload().clone())
}
