//! [`Shell`]: PdfCraft's app, plus what a window on Android needs that the desktop gets from its
//! window system.
//!
//! - **Close requests from Android** (Back, Esc): passed to PdfCraft as the window's close, so its
//!   "save changes?" question runs. Android finishes the activity only once PdfCraft lets the
//!   window close (see `android_main`).
//! - **Autosave when the window loses focus.** Android can end the app without asking (the close
//!   button of a DeX window, the system reclaiming memory), so the unsaved work is written to
//!   PdfCraft's crash recovery as soon as the user turns away; the next start offers it back.

use std::sync::atomic::{AtomicBool, Ordering};

use pdfcraft_ui_egui::PdfCraftApp;

/// Set by Android (Back, Esc); taken by the next frame.
static CLOSE_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Ask the window to close, as its close button would (from any thread).
pub fn request_close(ctx: Option<&egui::Context>) {
    CLOSE_REQUESTED.store(true, Ordering::SeqCst);
    if let Some(ctx) = ctx {
        ctx.request_repaint();
    }
}

pub struct Shell {
    app: PdfCraftApp,
    focused: bool,
}

impl Shell {
    pub fn new(app: PdfCraftApp) -> Self {
        Shell { app, focused: true }
    }
}

impl eframe::App for Shell {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if CLOSE_REQUESTED.swap(false, Ordering::SeqCst) {
            // egui turns this into the window's close request, which PdfCraft may hold back.
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        let focused = ctx.input(|i| i.focused);
        if self.focused && !focused {
            self.app.autosave_now();
        }
        self.focused = focused;
        self.app.logic(ctx, frame);
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.app.ui(ui, frame);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.app.save(storage);
    }

    fn on_exit(&mut self, gl: Option<&eframe::glow::Context>) {
        self.app.on_exit(gl);
    }

    fn auto_save_interval(&self) -> std::time::Duration {
        self.app.auto_save_interval()
    }

    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        self.app.clear_color(visuals)
    }

    fn persist_egui_memory(&self) -> bool {
        self.app.persist_egui_memory()
    }

    fn raw_input_hook(&mut self, ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        self.app.raw_input_hook(ctx, raw_input);
    }
}
