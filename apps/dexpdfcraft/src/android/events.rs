//! Events from the activity, and what the interface polls for them.

use std::sync::{Mutex, OnceLock, PoisonError};

use pdfcraft_ui_egui::OsEvent;

/// The interface's context, to wake it when something arrives.
static CONTEXT: OnceLock<egui::Context> = OnceLock::new();
/// Documents opened with the app ("Open with", "Share") not yet handed to the interface.
static OPENED: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Called once the interface exists.
pub fn set_context(ctx: egui::Context) {
    let _ = CONTEXT.set(ctx);
}

/// PdfCraft's `os_events` poll: the documents opened with the app since the last call.
pub fn poll() -> Vec<OsEvent> {
    let paths = std::mem::take(&mut *OPENED.lock().unwrap_or_else(PoisonError::into_inner));
    if paths.is_empty() { Vec::new() } else { vec![OsEvent::Open(paths)] }
}

/// Wake the interface (something was queued for its next frame).
pub fn wake() {
    if let Some(ctx) = CONTEXT.get() {
        ctx.request_repaint();
    }
}

pub fn dispatch(op: &str, json: &str) {
    match op {
        "picked" => match crate::protocol::parse_picked(json) {
            Some((id, paths)) => super::pickers::complete(id, paths),
            None => log::warn!("dexpdfcraft: malformed picked event"),
        },
        "open" => {
            let paths = crate::protocol::parse_open(json);
            log::info!("dexpdfcraft: {} document(s) opened with the app", paths.len());
            OPENED.lock().unwrap_or_else(PoisonError::into_inner).extend(paths);
            if let Some(ctx) = CONTEXT.get() {
                ctx.request_repaint();
            }
        }
        // Back or Esc: PdfCraft decides (it asks about unsaved changes first).
        "close" => crate::shell::request_close(CONTEXT.get()),
        _ => log::warn!("dexpdfcraft: unknown event {op}"),
    }
}
