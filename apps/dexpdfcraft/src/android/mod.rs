//! The Android side: the entry point and PdfCraft's backends for the system services
//! craft-android provides (pickers, clipboard, printing). Everything else (the JNI bridge, the
//! environment, the activity's input, the lifecycle) is craft-android's.

pub mod print;

use std::path::PathBuf;

use craft_android::android::{self as platform, AndroidApp, clipboard::Clip, pickers::PickKind};

/// Entry point, called by android-activity on the activity's own thread.
#[allow(unsafe_code)] // `no_mangle` is how android-activity's glue finds the entry point
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    let config = platform::Config {
        log_tag: "dexpdfcraft",
        // The OCR models and (when the APK carries Tesseract) its language data.
        asset_dirs: vec![("models", "PDFCRAFT_MODELS"), ("tessdata", "TESSDATA_PREFIX")],
    };
    let Some(started) = platform::start(&app, &config) else { return };
    platform::diag(&format!("DexPdfCraft {} (engine {})", env!("CARGO_PKG_VERSION"), pdfcraft_engine::build_info::VERSION));
    // The system pickers and clipboard behind PdfCraft's rfd and arboard (the fork's shims).
    rfd::set_backend(Box::new(SafPickers));
    arboard::set_backend(Box::new(SystemClipboard));
    let data = started.data_dir;
    let launch = crate::Launch {
        platform: crate::app::Platform { os: "android", source: "bundled sample".into() },
        files: Vec::new(),
        persistence_path: Some(data.join("config/app.ron")),
        sample: true,
        sample_marker: Some(data.join("state/sample-shown")),
    };
    let options_launch = launch.clone();
    crate::run(move |renderer| platform::native_options(&app, crate::native_options(renderer, &options_launch)), launch);
    // The window closed (PdfCraft allowed it, after asking about unsaved changes).
    platform::finish();
}

/// rfd's backend: the Storage Access Framework.
struct SafPickers;

impl rfd::Backend for SafPickers {
    fn pick(&self, kind: rfd::Kind, request: rfd::Request) -> rfd::Answer<Vec<PathBuf>> {
        let kind = match kind {
            rfd::Kind::OpenFile => PickKind::OpenFile,
            rfd::Kind::OpenFiles => PickKind::OpenFiles,
            rfd::Kind::Folder => PickKind::Folder,
            rfd::Kind::Folders => PickKind::Folders,
            rfd::Kind::Save => PickKind::Save,
        };
        platform::pickers::pick(kind, request.title.as_deref(), &request.filters, request.file_name.as_deref())
    }
}

/// arboard's backend: Android's clipboard.
struct SystemClipboard;

impl arboard::Backend for SystemClipboard {
    fn get(&self) -> Option<arboard::Content> {
        Some(match platform::clipboard::get()? {
            Clip::Text(text) => arboard::Content::Text(text),
            Clip::Html { html, text } => arboard::Content::Html { html, alt: Some(text) },
            Clip::Image { width, height, rgba } => arboard::Content::Image { width, height, bytes: rgba },
            Clip::Files(paths) => arboard::Content::Files(paths),
        })
    }

    fn set(&self, content: arboard::Content) -> Result<(), String> {
        platform::clipboard::set(match content {
            arboard::Content::Text(text) => Clip::Text(text),
            arboard::Content::Html { html, alt } => Clip::Html { html, text: alt.unwrap_or_default() },
            arboard::Content::Image { width, height, bytes } => Clip::Image { width, height, rgba: bytes },
            arboard::Content::Files(paths) => Clip::Files(paths),
        })
    }
}
