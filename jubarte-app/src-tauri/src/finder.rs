//! Finder → Jubarte. Two ways in, one path through:
//!
//! - "Open With → Jubarte" (and a drop on the Dock icon) arrives as
//!   `RunEvent::Opened`.
//! - Right-click → Quick Actions/Services → "Redline with Jubarte" or
//!   "Convert to PDF with Jubarte" are macOS Services (`NSServices` in
//!   `Info.plist`): AppKit hands the selected files to the provider in
//!   `finder_service.rs` on a pasteboard, and the service names the [`Intent`].
//!
//! Both end in [`deliver`]: stash the paths, then tell the window. The window
//! drains the stash itself (`take_pending_files`), so a launch that races the
//! webview delivers each file exactly once — a duplicate would run twice and
//! spend two of the free uses.

use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Runtime};

/// What a Finder service asked for. Open With and a Dock drop name none: the
/// window then redlines two files and keeps its mode for one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Intent {
    Redline,
    Convert,
}

/// Files handed over by Finder before the window took them, and what to make
/// of them.
#[derive(Debug, Default, Serialize)]
pub struct Pending {
    pub paths: Vec<String>,
    pub intent: Option<Intent>,
}

/// The undrained hand-over; drained by the `take_pending_files` command.
pub struct PendingFiles(pub Mutex<Pending>);

/// The window's cue to call `take_pending_files`.
pub const FILES_OPENED: &str = "files-opened";

/// `.docx` paths among `paths`, in order. Word's owner files (`~$name.docx`, the
/// lock file Word keeps beside an open document) are not documents.
pub fn docx_only<I: IntoIterator<Item = String>>(paths: I) -> Vec<String> {
    paths
        .into_iter()
        .filter(|p| {
            let name = std::path::Path::new(p)
                .file_name()
                .map(|n| n.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            name.ends_with(".docx") && !name.starts_with("~$")
        })
        .collect()
}

/// Queue `paths` for the window and bring it forward. A request for another
/// action than the undrained one replaces it: the last thing asked for wins.
pub fn deliver<R: Runtime>(app: &AppHandle<R>, paths: Vec<String>, intent: Option<Intent>) {
    let paths = docx_only(paths);
    if paths.is_empty() {
        return;
    }
    if let Some(pending) = app.try_state::<PendingFiles>() {
        let mut pending = pending.0.lock().unwrap();
        if pending.intent != intent {
            pending.paths.clear();
            pending.intent = intent;
        }
        pending.paths.extend(paths);
    }
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
    let _ = app.emit(FILES_OPENED, ());
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use tauri::test::{MockRuntime, mock_builder, mock_context, noop_assets};
    use tauri::{Listener, Manager};

    use super::{FILES_OPENED, Intent, Pending, PendingFiles, deliver, docx_only};

    fn app() -> tauri::App<MockRuntime> {
        mock_builder()
            .manage(PendingFiles(Mutex::new(Pending::default())))
            .build(mock_context(noop_assets()))
            .expect("mock app")
    }

    fn take_all(app: &tauri::App<MockRuntime>) -> Pending {
        std::mem::take(&mut *app.state::<PendingFiles>().0.lock().unwrap())
    }

    fn take(app: &tauri::App<MockRuntime>) -> Vec<String> {
        take_all(app).paths
    }

    #[test]
    fn each_service_names_what_the_window_should_make() {
        let app = app();
        deliver(
            app.handle(),
            vec!["/a/brief.docx".into()],
            Some(Intent::Convert),
        );
        let got = take_all(&app);
        assert_eq!(got.paths, ["/a/brief.docx"]);
        assert_eq!(got.intent, Some(Intent::Convert));

        deliver(
            app.handle(),
            vec!["/a/v1.docx".into(), "/a/v2.docx".into()],
            Some(Intent::Redline),
        );
        assert_eq!(take_all(&app).intent, Some(Intent::Redline));

        // Open With and a Dock drop say nothing: the window decides.
        deliver(app.handle(), vec!["/a/x.docx".into()], None);
        assert_eq!(take_all(&app).intent, None);
    }

    #[test]
    fn a_request_for_another_action_replaces_an_undrained_one() {
        let app = app();
        deliver(
            app.handle(),
            vec!["/a/v1.docx".into(), "/a/v2.docx".into()],
            Some(Intent::Redline),
        );
        deliver(
            app.handle(),
            vec!["/a/brief.docx".into()],
            Some(Intent::Convert),
        );
        let got = take_all(&app);
        assert_eq!(got.paths, ["/a/brief.docx"]);
        assert_eq!(got.intent, Some(Intent::Convert));

        // The same action, asked twice before the window drains, adds up.
        deliver(
            app.handle(),
            vec!["/a/a.docx".into()],
            Some(Intent::Convert),
        );
        deliver(
            app.handle(),
            vec!["/a/b.docx".into()],
            Some(Intent::Convert),
        );
        assert_eq!(take(&app), ["/a/a.docx", "/a/b.docx"]);
    }

    #[test]
    fn the_window_reads_the_intent_in_lowercase() {
        let json = serde_json::to_string(&Pending {
            paths: vec!["/a/x.docx".into()],
            intent: Some(Intent::Convert),
        })
        .unwrap();
        assert_eq!(json, r#"{"paths":["/a/x.docx"],"intent":"convert"}"#);
        let none = serde_json::to_string(&Pending::default()).unwrap();
        assert_eq!(none, r#"{"paths":[],"intent":null}"#);
    }

    #[test]
    fn deliver_queues_the_documents_and_cues_the_window_once() {
        let app = app();
        let cues = Arc::new(AtomicUsize::new(0));
        let seen = cues.clone();
        app.listen_any(FILES_OPENED, move |_| {
            seen.fetch_add(1, Ordering::SeqCst);
        });

        deliver(app.handle(), vec!["/a/notes.pdf".into()], None);
        assert!(take(&app).is_empty(), "nothing to queue");
        assert_eq!(cues.load(Ordering::SeqCst), 0, "no cue without a document");

        deliver(
            app.handle(),
            vec![
                "/a/old.docx".into(),
                "/a/~$old.docx".into(),
                "/a/new.docx".into(),
            ],
            None,
        );
        assert_eq!(cues.load(Ordering::SeqCst), 1);
        assert_eq!(take(&app), ["/a/old.docx", "/a/new.docx"]);
        assert!(
            take(&app).is_empty(),
            "a drained pair is not delivered again"
        );
    }

    #[test]
    fn keeps_docx_in_order_and_case_insensitively() {
        let got = docx_only(
            ["/a/Old.docx", "/a/notes.pdf", "/a/NEW.DOCX", "/a/x.docxx"].map(String::from),
        );
        assert_eq!(got, ["/a/Old.docx", "/a/NEW.DOCX"]);
    }

    #[test]
    fn drops_word_owner_lock_files() {
        let got = docx_only(["/a/~$ntract.docx", "/a/contract.docx"].map(String::from));
        assert_eq!(got, ["/a/contract.docx"]);
    }

    #[test]
    fn a_folder_named_like_a_docx_parent_does_not_count() {
        assert!(docx_only(["/a/b.docx/readme.txt".to_string()]).is_empty());
    }
}
