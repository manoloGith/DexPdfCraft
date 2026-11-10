//! DexPdfCraft: PdfCraft's desktop interface on Android, for Samsung DeX and Android's desktop
//! windowing. The Android app (`android/`) is a `NativeActivity` that loads this library and
//! calls [`android_main`]; egui draws the whole window with wgpu (Vulkan or GLES), or with OpenGL
//! ES through glow when wgpu can't start.
//!
//! Phase 1 (this spike) shows [`app::ProbeApp`]; phase 2 runs `pdfcraft-ui-egui` instead.

pub mod app;
pub mod doc;

/// The sample opened when no document is given (a form from upstream's test data).
pub const SAMPLE_PDF: &[u8] = include_bytes!("../assets/sample.pdf");

/// The window's options, shared by Android and the desktop binary.
pub fn native_options(renderer: eframe::Renderer) -> eframe::NativeOptions {
    eframe::NativeOptions {
        renderer,
        viewport: egui::ViewportBuilder::default().with_title("DexPdfCraft").with_inner_size([1280.0, 800.0]),
        ..Default::default()
    }
}

/// Run the window with wgpu, then with glow if wgpu can't start (old or unusual GPU drivers).
pub fn run(mut options: impl FnMut(eframe::Renderer) -> eframe::NativeOptions, platform: app::Platform, pdf: Result<(String, Vec<u8>), String>) {
    let mut last_error = None;
    for renderer in [eframe::Renderer::Wgpu, eframe::Renderer::Glow] {
        let (platform, pdf) = (platform.clone(), pdf.clone());
        let result = eframe::run_native(
            "DexPdfCraft",
            options(renderer),
            Box::new(move |cc| Ok(Box::new(app::ProbeApp::new(cc, platform, pdf)))),
        );
        match result {
            Ok(()) => return,
            Err(e) => {
                log::error!("dexpdfcraft: the {renderer} renderer could not start: {e}");
                last_error = Some(e);
            }
        }
    }
    if let Some(e) = last_error {
        log::error!("dexpdfcraft: no renderer could start: {e}");
    }
}

#[cfg(target_os = "android")]
mod android {
    use winit::platform::android::activity::AndroidApp;

    /// Entry point, called by android-activity on the activity's own thread.
    #[allow(unsafe_code)] // `no_mangle` is how NativeActivity's glue finds the entry point
    #[unsafe(no_mangle)]
    fn android_main(app: AndroidApp) {
        android_logger::init_once(android_logger::Config::default().with_max_level(log::LevelFilter::Info).with_tag("dexpdfcraft"));
        log::info!("dexpdfcraft: starting, engine {}", pdfcraft_engine::build_info::VERSION);
        let platform = crate::app::Platform { os: "android", source: "bundled sample".into() };
        let pdf = Ok(("sample.pdf".to_owned(), crate::SAMPLE_PDF.to_vec()));
        crate::run(
            move |renderer| {
                let mut options = crate::native_options(renderer);
                options.android_app = Some(app.clone());
                options
            },
            platform,
            pdf,
        );
    }
}
