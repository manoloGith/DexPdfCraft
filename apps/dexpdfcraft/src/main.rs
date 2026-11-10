//! DexPdfCraft in a desktop window, to iterate without a device:
//! `cargo run -p dexpdfcraft --features desktop -- [file.pdf…]` (add `probe` for the diagnostics window).

fn main() {
    let files: Vec<String> = std::env::args().skip(1).collect();
    let launch = dexpdfcraft::Launch {
        platform: dexpdfcraft::app::Platform { os: std::env::consts::OS, source: "command line".into() },
        sample: files.is_empty(),
        files,
        persistence_path: None,
    };
    let options_launch = launch.clone();
    dexpdfcraft::run(move |renderer| dexpdfcraft::native_options(renderer, &options_launch), launch);
}
