//! The spike in a desktop window: `cargo run -p dexpdfcraft --features desktop -- [file.pdf]`.

fn main() {
    let path = std::env::args().nth(1);
    let pdf = match &path {
        Some(p) => std::fs::read(p).map(|b| (p.clone(), b)).map_err(|e| format!("{p}: {e}")),
        None => Ok(("sample.pdf".to_owned(), dexpdfcraft::SAMPLE_PDF.to_vec())),
    };
    let platform = dexpdfcraft::app::Platform { os: std::env::consts::OS, source: path.unwrap_or_else(|| "bundled sample".into()) };
    dexpdfcraft::run(dexpdfcraft::native_options, platform, pdf);
}
