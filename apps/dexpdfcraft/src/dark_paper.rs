//! Dark paper for PdfCraft: in the dark theme, pages are drawn on craft-android's dark paper
//! ([`craft_android::paper`]) through PdfCraft's reading colours (Preferences ▸ Reading
//! colours). Dark paints become light grey and light ones dark grey, line art included, so white
//! boxes and table fills turn dark with the page; images and photos keep their colours (PdfCraft
//! never recolours them). Back to off when the theme turns light. Reading colours the user picks
//! are left alone ([`craft_android::paper::on_theme_change`]).

use craft_android::paper;
use pdfcraft_render::{ReadingColors, ReadingParams};
use pdfcraft_ui_egui::PdfCraftApp;

/// craft-android's dark paper as PdfCraft reading colours.
pub const DARK_PAPER: ReadingColors =
    ReadingColors { params: Some(ReadingParams { text: paper::LIGHT_INK, background: paper::DARK_PAPER, only_black: false, line_art: true }) };

/// The interface turned dark (or light): [`craft_android::Shell::on_theme_change`].
pub fn on_theme_change(app: &mut PdfCraftApp, dark: bool) {
    if let Some(colors) = paper::on_theme_change(dark, &app.reading_colors, &ReadingColors::default(), &DARK_PAPER) {
        app.set_reading_colors(colors);
    }
}
