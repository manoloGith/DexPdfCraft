//! Printing through Android's print service.
//!
//! PdfCraft's Print dialog composes the print-ready PDF (pages, scaling, booklets, n-up, marks)
//! and hands it to a spooler. Android has no spooler PdfCraft can talk to, so the dialog offers
//! one "printer", Android printing, whose spooler writes that PDF to the cache and asks
//! MainActivity to open the system print dialog with it. There the user picks the printer
//! (Wi-Fi, Mopria, the manufacturer's service, Save as PDF), copies and two-sided printing.

use std::path::PathBuf;
use std::sync::Arc;

use pdfcraft_engine::print::spool;
use pdfcraft_ui_egui::PdfCraftApp;

/// The printer PdfCraft's dialog lists (the system dialog picks the real one).
const ANDROID_PRINTER: &str = "Android";

/// Send PdfCraft's print jobs to Android's print dialog, writing them in `dir`.
pub fn install(app: &mut PdfCraftApp, dir: PathBuf) {
    app.printer_source = Some(Arc::new(|| {
        vec![spool::Printer {
            name: ANDROID_PRINTER.to_owned(),
            description: "Android printing (choose the printer next)".to_owned(),
            default: true,
            two_sided: None,
        }]
    }));
    // No driver options here: the system dialog has them.
    app.options_source = Some(Arc::new(|_| Vec::new()));
    app.two_sided_source = Some(Arc::new(|_| None));
    app.spooler = Some(Arc::new(move |bytes: &[u8], job: &spool::Job| {
        let title = if job.title.trim().is_empty() { "PdfCraft" } else { job.title.as_str() };
        craft_android::android::diag(&format!("print: {} copies", job.copies));
        craft_android::android::print::print_pdf(bytes, title, &dir)
    }));
}
