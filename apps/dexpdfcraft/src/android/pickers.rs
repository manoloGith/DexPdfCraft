//! The rfd shim's backend: the Storage Access Framework, through MainActivity.
//!
//! A picker request gets an id and a channel; the activity shows the system picker, copies what
//! was chosen to local mirrors (DocumentStore.kt) and answers with a `picked` event carrying the
//! id and the local paths. PdfCraft waits for the answer on its picker thread, as on the desktop.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Sender, channel};
use std::sync::{Mutex, PoisonError};

use crate::protocol::{PickKind, pick_request};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);
static WAITING: Mutex<Option<HashMap<u64, Sender<Vec<PathBuf>>>>> = Mutex::new(None);

pub struct SafPickers;

impl rfd::Backend for SafPickers {
    fn pick(&self, kind: rfd::Kind, request: rfd::Request) -> rfd::Answer<Vec<PathBuf>> {
        let kind = match kind {
            rfd::Kind::OpenFile => PickKind::OpenFile,
            rfd::Kind::OpenFiles => PickKind::OpenFiles,
            rfd::Kind::Folder => PickKind::Folder,
            rfd::Kind::Folders => PickKind::Folders,
            rfd::Kind::Save => PickKind::Save,
        };
        if !super::bridge::available() {
            log::warn!("dexpdfcraft: no activity for the picker");
            return Box::pin(async { Vec::new() });
        }
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = channel();
        WAITING.lock().unwrap_or_else(PoisonError::into_inner).get_or_insert_with(HashMap::new).insert(id, tx);
        let json = pick_request(id, kind, request.title.as_deref(), &request.filters, request.file_name.as_deref());
        super::bridge::call("pick", &json, None);
        // Awaited on PdfCraft's picker thread (pollster), which may block until the answer.
        Box::pin(async move { rx.recv().unwrap_or_default() })
    }
}

/// The activity's answer to request `id`.
pub fn complete(id: u64, paths: Vec<PathBuf>) {
    let sender = WAITING.lock().unwrap_or_else(PoisonError::into_inner).as_mut().and_then(|w| w.remove(&id));
    match sender {
        Some(tx) => {
            let _ = tx.send(paths);
        }
        None => log::warn!("dexpdfcraft: an answer for unknown picker {id}"),
    }
}

/// A new activity: pickers the old one was showing will never be answered, so cancel them (PdfCraft
/// shows one picker at a time and would otherwise wait for ever).
pub fn cancel_all() {
    let waiting = WAITING.lock().unwrap_or_else(PoisonError::into_inner).take().unwrap_or_default();
    for (_, tx) in waiting {
        let _ = tx.send(Vec::new());
    }
}
