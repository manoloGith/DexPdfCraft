//! The Android side: the entry point, the environment PdfCraft expects, and the bridge to the
//! activity (pickers, clipboard, "Open with").

mod bridge;
mod clipboard;
pub mod events;
mod pickers;

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
    // The system pickers and clipboard, through MainActivity (rfd and arboard shims).
    rfd::set_backend(Box::new(pickers::SafPickers));
    arboard::set_backend(Box::new(clipboard::SystemClipboard));
    let launch = crate::Launch {
        platform: crate::app::Platform { os: "android", source: "bundled sample".into() },
        files: Vec::new(),
        persistence_path: Some(data.join("config/app.ron")),
        sample: true,
        sample_marker: Some(data.join("state/sample-shown")),
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
    // The window closed (PdfCraft allowed it, after asking about unsaved changes): close the
    // activity too. Its onDestroy ends the process, so the next start gets a fresh one (winit's
    // event loop can be created only once per process).
    log::info!("dexpdfcraft: the window closed; finishing the activity");
    bridge::call("finish", "", None);
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
