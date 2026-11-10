//! Starting PdfCraft's interface the way the desktop app does (`upstream/apps/pdfcraft`), minus
//! what Android has no use for: the single-instance socket, the control channel, Apple events,
//! system printers (Android printing comes in phase 3) and update checks (the store updates).

use pdfcraft_ui_egui::PdfCraftApp;

use crate::Launch;

/// eframe storage key, as on the desktop.
const STORAGE_KEY: &str = "pdfcraft";
/// Set once the first start has opened the sample, so later starts don't.
const SAMPLE_SHOWN_KEY: &str = "dexpdfcraft.sample_shown";

pub fn create(cc: &eframe::CreationContext<'_>, launch: &Launch) -> PdfCraftApp {
    let mut app = PdfCraftApp::new();
    app.units = pdfcraft_ui_egui::units::Unit::system();
    if let Some(json) = cc.storage.and_then(|s| s.get_string(STORAGE_KEY)) {
        app.restore(&json);
    }
    app.window_state = pdfcraft_ui_egui::window_state::WindowState::started_with(cc.storage);
    app.set_startup_update_check(false);
    if let Some(dir) = pdfcraft_ui_egui::RecoveryStore::default_dir() {
        app.enable_recovery(pdfcraft_ui_egui::RecoveryStore::new(dir));
    }
    app.ocr_verify.dictionary_dir = pdfcraft_ui_egui::default_dictionary_dir();
    match &cc.wgpu_render_state {
        Some(state) => {
            let adapter = state.adapter.get_info();
            log::info!("dexpdfcraft: renderer wgpu, {} ({:?}, {:?})", adapter.name, adapter.device_type, adapter.backend);
        }
        None => log::info!("dexpdfcraft: renderer OpenGL ES (glow)"),
    }
    app.reopen_last_files(&launch.files);
    for file in &launch.files {
        app.open_path(file);
    }
    let first_start = cc.storage.and_then(|s| s.get_string(SAMPLE_SHOWN_KEY)).is_none();
    if launch.sample && launch.files.is_empty() && first_start {
        if let Ok(mut inbox) = app.startup_inbox.lock() {
            inbox.push(("DexPdfCraft sample.pdf".to_owned(), crate::SAMPLE_PDF.to_vec()));
        }
    }
    app
}
