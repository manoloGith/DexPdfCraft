//! The Android side: the entry point, the environment PdfCraft expects, and the bridge to the
//! activity (pickers, clipboard, "Open with").

mod bridge;
mod clipboard;
pub mod desktop;
pub mod print;
pub mod events;
mod pickers;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use winit::platform::android::activity::AndroidApp;
use winit::platform::android::activity::input::Axis;

/// `android_main` has run in this process. winit's event loop can be created once per process,
/// so a second activity in the same process can't start the window (MainActivity ends the
/// process when its activity is destroyed; this guards the rest).
static STARTED: AtomicBool = AtomicBool::new(false);

/// What a panic leaves for the next start to show (MainActivity reads it).
const CRASH_FILE: &str = "crash.txt";

/// The diagnostics log MainActivity keeps (lifecycle, first mouse events); the library adds its
/// own milestones, so a report from a device without adb tells what happened.
const DIAG_FILE: &str = "diag.log";
static DIAG: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

/// Append a line to the diagnostics log (and logcat).
pub fn diag(line: &str) {
    log::info!("dexpdfcraft: {line}");
    let Some(path) = DIAG.get() else { return };
    use std::io::Write as _;
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
        let _ = writeln!(f, "{now} rust: {line}");
    }
}

/// Entry point, called by android-activity on the activity's own thread.
#[allow(unsafe_code)] // `no_mangle` is how android-activity's glue finds the entry point
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    android_logger::init_once(android_logger::Config::default().with_max_level(log::LevelFilter::Info).with_tag("dexpdfcraft"));
    log::info!("dexpdfcraft: starting, engine {}", pdfcraft_engine::build_info::VERSION);
    let data = app.internal_data_path().unwrap_or_else(|| PathBuf::from("/data/local/tmp/dexpdfcraft"));
    let _ = DIAG.set(data.join(DIAG_FILE));
    record_panics(data.join(CRASH_FILE));
    if STARTED.swap(true, Ordering::SeqCst) {
        // MainActivity restarts the app in a fresh process (RestartActivity).
        diag("android_main called again in the same process; not starting");
        return;
    }
    diag(&format!("android_main starts (engine {})", pdfcraft_engine::build_info::VERSION));
    // GameActivity copies only X and Y from motion events unless asked: the wheel's axes.
    app.enable_motion_axis(Axis::Vscroll);
    app.enable_motion_axis(Axis::Hscroll);
    let config = app.config();
    let locale = crate::posix_locale(config.language().as_deref(), config.country().as_deref());
    set_environment(&data, &locale);
    // The OCR models ship in the APK's assets; PdfCraft reads them from files.
    if let Some(dir) = extract_models(&app, &data.join("models")) {
        set_var("PDFCRAFT_MODELS", dir.as_os_str());
    }
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
    diag("the event loop ended; finishing the activity");
    bridge::call("finish", "", None);
}

/// The OCR models the APK carries (assets/models), copied to `dir` the first time and after an
/// update that changed them (their size differs). The folder, if a pair is there.
fn extract_models(app: &AndroidApp, dir: &Path) -> Option<PathBuf> {
    const PAIRS: [[&str; 2]; 2] = [["text-detection.onnx", "text-recognition.onnx"], ["text-detection.rten", "text-recognition.rten"]];
    let assets = app.asset_manager();
    if let Err(e) = std::fs::create_dir_all(dir) {
        diag(&format!("models: {e}"));
        return None;
    }
    for pair in PAIRS {
        let mut complete = true;
        for name in pair {
            let Ok(asset_name) = std::ffi::CString::new(format!("models/{name}")) else { return None };
            let Some(mut asset) = assets.open(&asset_name) else {
                complete = false;
                break;
            };
            let target = dir.join(name);
            let current = std::fs::metadata(&target).map(|m| m.len()).ok();
            if current == u64::try_from(asset.length()).ok() {
                continue;
            }
            let partial = dir.join(format!(".{name}.part"));
            let copied = std::fs::File::create(&partial)
                .and_then(|mut f| std::io::copy(&mut asset, &mut f))
                .and_then(|_| std::fs::rename(&partial, &target));
            if let Err(e) = copied {
                let _ = std::fs::remove_file(&partial);
                diag(&format!("models: copying {name} failed: {e}"));
                complete = false;
                break;
            }
            diag(&format!("models: {name} installed"));
        }
        if complete {
            return Some(dir.to_path_buf());
        }
    }
    diag("models: none in the APK (Scan & OCR will say they aren't installed)");
    None
}

/// Write every panic (message, place, backtrace) to `file`, after the default hook has logged it,
/// so the next start can show it to the user (MainActivity), who may have no adb.
fn record_panics(file: PathBuf) {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        default(info);
        let report = format!(
            "DexPdfCraft {} (engine {})\n{info}\n\n{}",
            env!("CARGO_PKG_VERSION"),
            pdfcraft_engine::build_info::VERSION,
            std::backtrace::Backtrace::force_capture()
        );
        let _ = std::fs::write(&file, &report);
        diag(&format!("panic: {info}"));
    }));
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
