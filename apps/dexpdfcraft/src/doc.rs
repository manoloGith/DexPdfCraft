//! One open PDF, drawn with PdfCraft's own renderer (`pdfcraft-render`, the same CPU rasterizer as
//! the desktop app). The spike renders on the UI thread; the real app uses upstream's render pool.

use std::sync::Arc;

use pdfcraft_render::{DocInfo, PageRenderer, RenderConfig, RenderRequest, RequestKind, effective_scale, inspect};

/// A rendered page, as egui pixels.
pub struct Raster {
    pub page: usize,
    pub scale: f32,
    pub image: egui::ColorImage,
    pub millis: u32,
}

pub struct Doc {
    pub name: String,
    pub info: DocInfo,
    renderer: PageRenderer,
}

impl Doc {
    /// Parse `bytes` (no password: the spike opens only the bundled sample and plain files).
    pub fn open(name: impl Into<String>, bytes: Vec<u8>) -> Result<Self, String> {
        let bytes = Arc::new(bytes);
        let info = inspect(bytes.clone(), None).map_err(|e| e.to_string())?;
        if info.pages.is_empty() {
            return Err("the document has no pages".into());
        }
        let renderer = PageRenderer::new(bytes, RenderConfig::default());
        Ok(Doc { name: name.into(), info, renderer })
    }

    pub fn page_count(&self) -> usize {
        self.info.pages.len()
    }

    /// Displayed size of `page` in PDF points (after `/Rotate`).
    pub fn page_size(&self, page: usize) -> egui::Vec2 {
        self.info.pages.get(page).map_or(egui::vec2(612.0, 792.0), |p| egui::vec2(p.width, p.height))
    }

    /// Draw `page` at `scale` device pixels per PDF point (capped to the renderer's limits).
    pub fn render(&mut self, page: usize, scale: f32) -> Result<Raster, String> {
        let size = self.page_size(page);
        let scale = effective_scale(size.x, size.y, scale);
        let out = self.renderer.render(RenderRequest { page, kind: RequestKind::Pixels, tile: None, scale, tag: 0, preview: false });
        if let Some(e) = out.error {
            return Err(format!("page {} could not be drawn: {e}", page.saturating_add(1)));
        }
        let image = egui::ColorImage::from_rgba_premultiplied([out.width as usize, out.height as usize], out.rgba.as_ref());
        Ok(Raster { page, scale, image, millis: out.millis })
    }
}
