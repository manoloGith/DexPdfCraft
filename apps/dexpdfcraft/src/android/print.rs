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
use serde_json::json;

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
        if !super::bridge::available() {
            return Err("printing is not available yet".to_owned());
        }
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        // One file per job; the activity removes it when the system has taken it.
        let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
        let path = dir.join(format!("print-{stamp}.pdf"));
        std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
        let title = if job.title.trim().is_empty() { "PdfCraft".to_owned() } else { job.title.clone() };
        super::diag(&format!("print: {} bytes, {} copies", bytes.len(), job.copies));
        super::bridge::call("print", &json!({ "path": path.to_string_lossy(), "title": title }).to_string(), None);
        Ok("Sent to Android's print dialog".to_owned())
    }));
}
