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
    Box::new(pdfcraft::create(cc, &launch))
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
mod android {
    use std::path::{Path, PathBuf};

    use winit::platform::android::activity::AndroidApp;

    /// Entry point, called by android-activity on the activity's own thread.
    #[allow(unsafe_code)] // `no_mangle` is how android-activity's glue finds the entry point
    #[unsafe(no_mangle)]
    fn android_main(app: AndroidApp) {
        android_logger::init_once(android_logger::Config::default().with_max_level(log::LevelFilter::Info).with_tag("dexpdfcraft"));
        log::info!("dexpdfcraft: starting, engine {}", pdfcraft_engine::build_info::VERSION);
        let data = app.internal_data_path().unwrap_or_else(|| PathBuf::from("/data/local/tmp/dexpdfcraft"));
        let config = app.config();
        let locale = crate::posix_locale(config.language().as_deref(), config.country().as_deref());
        set_environment(&data, &locale);
        let launch = crate::Launch {
            platform: crate::app::Platform { os: "android", source: "bundled sample".into() },
            files: Vec::new(),
            persistence_path: Some(data.join("config/app.ron")),
            sample: true,
        };
        let options_launch = launch.clone();
        crate::run(
            move |renderer| {
                let mut options = crate::native_options(renderer, &options_launch);
                options.android_app = Some(app.clone());
                options
            },
            launch,
        );
    }

    /// PdfCraft keeps settings, recovery data and its dictionary under `$HOME` and the XDG folders
    /// on Linux-like systems, and takes its Auto language from `LANG`; an Android process has
    /// none of them. Point the folders at the app's private storage, so autosave and crash
    /// recovery work unchanged, and `LANG` at the system language.
    fn set_environment(data: &Path, locale: &str) {
        let vars = [
            ("HOME", data.to_path_buf()),
            ("XDG_DATA_HOME", data.join("share")),
            ("XDG_CONFIG_HOME", data.join("config")),
            ("XDG_CACHE_HOME", data.join("cache")),
            ("XDG_STATE_HOME", data.join("state")),
            ("TMPDIR", data.join("tmp")),
        ];
        for (key, dir) in vars {
            if let Err(e) = std::fs::create_dir_all(&dir) {
                log::warn!("dexpdfcraft: {}: {e}", dir.display());
            }
            set_var(key, dir.as_os_str());
        }
        log::info!("dexpdfcraft: locale {locale}");
        set_var("LANG", std::ffi::OsStr::new(locale));
    }

    fn set_var(key: &str, value: &std::ffi::OsStr) {
        // SAFETY: called first thing in `android_main`, before this library starts any thread that
        // reads the environment (eframe, the render pool and PdfCraft start later).
        #[allow(unsafe_code)]
        unsafe {
            std::env::set_var(key, value);
        }
    }
}
