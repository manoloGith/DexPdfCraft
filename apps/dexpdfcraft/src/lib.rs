//! DexPdfCraft: PdfCraft's desktop interface on Android, for Samsung DeX and Android's desktop
//! windowing. The Android app (`android/`) is a `GameActivity` that loads this library and
//! calls `android_main`; egui draws the whole window with wgpu (Vulkan or GLES), or with OpenGL
//! ES through glow when wgpu can't start.
//!
//! The window is PdfCraft's own interface ([`pdfcraft_ui_egui::PdfCraftApp`]), as on the desktop.
//! The `probe` feature starts the phase 1 diagnostics window ([`app::ProbeApp`]) instead.

pub mod app;
pub mod doc;
pub mod pdfcraft;
pub mod ime;
pub mod insets;
pub mod keyboard;
pub mod pointer;
pub mod protocol;
pub mod shell;

/// The sample opened on first start (a test document of DexPdfCraft's own).
pub const SAMPLE_PDF: &[u8] = include_bytes!("../assets/sample.pdf");

/// What to start with.
#[derive(Clone, Debug)]
pub struct Launch {
    pub platform: app::Platform,
    /// Files to open (paths).
    pub files: Vec<String>,
    /// Where eframe keeps settings (`app.ron`).
    pub persistence_path: Option<std::path::PathBuf>,
    /// Open the bundled sample when nothing else opens.
    pub sample: bool,
    /// With `sample`: show it only if this file doesn't exist, and create it then, so only the
    /// first start shows it. `None`: every start.
    pub sample_marker: Option<std::path::PathBuf>,
}

/// The window's options, shared by Android and the desktop binary.
pub fn native_options(renderer: eframe::Renderer, launch: &Launch) -> eframe::NativeOptions {
    let mut options = eframe::NativeOptions {
        renderer,
        viewport: egui::ViewportBuilder::default()
            .with_title("DexPdfCraft")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size(pdfcraft_ui_egui::MIN_WINDOW_SIZE)
            .with_drag_and_drop(true),
        persistence_path: launch.persistence_path.clone(),
        ..Default::default()
    };
    // As on the desktop: PdfCraft picks its own last-resort system face.
    options.system_font_fallback = false;
    options
}

/// Run the window with wgpu, then with glow if wgpu can't start (old or unusual GPU drivers).
pub fn run(mut options: impl FnMut(eframe::Renderer) -> eframe::NativeOptions, launch: Launch) {
    let mut last_error = None;
    for renderer in [eframe::Renderer::Wgpu, eframe::Renderer::Glow] {
        let launch = launch.clone();
        let result = eframe::run_native("DexPdfCraft", options(renderer), Box::new(move |cc| Ok(create(cc, launch))));
        match result {
            Ok(()) => return,
            Err(e) => {
                log::error!("dexpdfcraft: the {renderer} renderer could not start: {e}");
                #[cfg(target_os = "android")]
                android::diag(&format!("the {renderer} renderer could not start: {e}"));
                last_error = Some(e);
            }
        }
    }
    if let Some(e) = last_error {
        log::error!("dexpdfcraft: no renderer could start: {e}");
    }
}

fn create(cc: &eframe::CreationContext<'_>, launch: Launch) -> Box<dyn eframe::App> {
    if cfg!(feature = "probe") {
        let pdf = Ok(("sample.pdf".to_owned(), SAMPLE_PDF.to_vec()));
        return Box::new(app::ProbeApp::new(cc, launch.platform, pdf));
    }
    Box::new(shell::Shell::new(pdfcraft::create(cc, &launch)))
}

/// The system language as a POSIX locale (`es_ES.UTF-8`), which PdfCraft's Auto language reads
/// from `LANG` as on Linux. `None` parts give `C.UTF-8` (English).
pub fn posix_locale(language: Option<&str>, country: Option<&str>) -> String {
    let clean = |s: &str| s.chars().filter(char::is_ascii_alphabetic).collect::<String>();
    match (language.map(clean).filter(|l| !l.is_empty()), country.map(clean).filter(|c| !c.is_empty())) {
        (Some(l), Some(c)) => format!("{}_{}.UTF-8", l.to_ascii_lowercase(), c.to_ascii_uppercase()),
        (Some(l), None) => format!("{}.UTF-8", l.to_ascii_lowercase()),
        _ => "C.UTF-8".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::posix_locale;

    #[test]
    fn the_android_locale_becomes_a_posix_one() {
        assert_eq!(posix_locale(Some("es"), Some("ES")), "es_ES.UTF-8");
        assert_eq!(posix_locale(Some("pt"), Some("br")), "pt_BR.UTF-8");
        assert_eq!(posix_locale(Some("de"), None), "de.UTF-8");
        assert_eq!(posix_locale(None, Some("US")), "C.UTF-8");
        assert_eq!(posix_locale(Some("e;s"), Some("")), "es.UTF-8");
    }
}

#[cfg(target_os = "android")]
pub(crate) mod android;
