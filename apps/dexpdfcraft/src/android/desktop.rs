//! What a desktop window shows that egui asks winit for, and winit ignores on Android: the
//! mouse cursor's shape and the window's title. Sent to MainActivity only when they change.

use std::sync::{Mutex, PoisonError};

use serde_json::json;

static CURSOR: Mutex<Option<i32>> = Mutex::new(None);
static TITLE: Mutex<Option<String>> = Mutex::new(None);

/// The cursor egui wants this frame.
pub fn set_cursor(icon: egui::CursorIcon) {
    let kind = crate::pointer::pointer_icon_type(icon);
    let mut last = CURSOR.lock().unwrap_or_else(PoisonError::into_inner);
    if *last != Some(kind) && super::bridge::available() {
        *last = Some(kind);
        super::bridge::call("cursor", &json!({ "type": kind }).to_string(), None);
    }
}

/// The window's title (PdfCraft names the document in front).
pub fn set_title(title: &str) {
    let mut last = TITLE.lock().unwrap_or_else(PoisonError::into_inner);
    if last.as_deref() != Some(title) && super::bridge::available() {
        *last = Some(title.to_owned());
        super::bridge::call("title", &json!({ "title": title }).to_string(), None);
    }
}
