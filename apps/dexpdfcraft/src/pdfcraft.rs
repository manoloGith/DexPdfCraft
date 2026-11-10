//! Starting PdfCraft's interface the way the desktop app does (`upstream/apps/pdfcraft`), minus
//! what Android has no use for: the single-instance socket, the control channel, Apple events,
//! system printers (Android printing is still to come) and update checks (the store updates).

use pdfcraft_ui_egui::PdfCraftApp;

use crate::Launch;

/// eframe storage key, as on the desktop.
const STORAGE_KEY: &str = "pdfcraft";

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
            let line = format!("renderer wgpu, {} ({:?}, {:?})", adapter.name, adapter.device_type, adapter.backend);
            log::info!("dexpdfcraft: {line}");
            #[cfg(target_os = "android")]
            crate::android::diag(&line);
        }
        None => {
            log::info!("dexpdfcraft: renderer OpenGL ES (glow)");
            #[cfg(target_os = "android")]
            crate::android::diag("renderer OpenGL ES (glow)");
        }
    }
    // "Open with" and "Share" from other apps arrive as the desktop's OS events do.
    #[cfg(target_os = "android")]
    {
        crate::android::events::set_context(cc.egui_ctx.clone());
        app.os_events = Some(Box::new(crate::android::events::poll));
        // Print jobs go to Android's print dialog; the spool files live in the cache.
        let cache = std::env::var_os("XDG_CACHE_HOME").map_or_else(std::env::temp_dir, std::path::PathBuf::from);
        crate::android::print::install(&mut app, cache.join("print"));
    }
    app.reopen_last_files(&launch.files);
    for file in &launch.files {
        app.open_path(file);
    }
    let first_time = match &launch.sample_marker {
        None => true,
        Some(marker) if marker.exists() => false,
        Some(marker) => {
            // Only the first start shows the sample.
            if let Some(dir) = marker.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            if let Err(e) = std::fs::write(marker, b"") {
                log::warn!("dexpdfcraft: {}: {e}", marker.display());
            }
            true
        }
    };
    if launch.sample
        && launch.files.is_empty()
        && first_time
        && let Ok(mut inbox) = app.startup_inbox.lock()
    {
        inbox.push(("DexPdfCraft sample.pdf".to_owned(), crate::SAMPLE_PDF.to_vec()));
    }
    app
}
