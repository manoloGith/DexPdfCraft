//! Phase 1 spike: a window laid out like PdfCraft's desktop app (menu bar from the engine's command
//! registry, quick toolbar, All tools on the left, the page in the middle, a status bar) that
//! checks what DeX gives an egui app: the pointer (hover, right and middle buttons), the wheel,
//! Ctrl+wheel zoom, keyboard shortcuts and typed text, window resizes, density and the renderer.
//!
//! Phase 2 replaces this window with the real `pdfcraft-ui-egui` interface.

use std::collections::VecDeque;

use pdfcraft_engine::commands::{COMMANDS, CommandSpec};

use crate::doc::{Doc, Raster};

const MENUS: [&str; 5] = ["File", "Edit", "View", "Pages", "Help"];
const LOG_LINES: usize = 200;
const ZOOM_STEPS: [f32; 12] = [0.25, 0.33, 0.5, 0.67, 0.75, 1.0, 1.25, 1.5, 2.0, 3.0, 4.0, 6.0];

/// What the window was started with, shown in the status bar and the diagnostics.
#[derive(Clone, Debug, Default)]
pub struct Platform {
    /// "android" or the desktop OS.
    pub os: &'static str,
    /// Where the document came from (bundled sample, a path…).
    pub source: String,
}

pub struct ProbeApp {
    platform: Platform,
    doc: Option<Doc>,
    error: Option<String>,
    page: usize,
    /// PDF points to egui points.
    zoom: f32,
    raster: Option<Raster>,
    texture: Option<egui::TextureHandle>,
    log: VecDeque<String>,
    typed: String,
    left_open: bool,
    right_open: bool,
    ui_scale: f32,
    frames: u64,
    last_size: egui::Vec2,
    renderer_name: String,
}

impl ProbeApp {
    pub fn new(cc: &eframe::CreationContext<'_>, platform: Platform, pdf: Result<(String, Vec<u8>), String>) -> Self {
        let renderer_name = renderer_name(cc);
        log::info!("dexpdfcraft: renderer {renderer_name}");
        let (doc, error) = match pdf.and_then(|(name, bytes)| Doc::open(name, bytes)) {
            Ok(d) => (Some(d), None),
            Err(e) => (None, Some(e)),
        };
        ProbeApp {
            platform,
            doc,
            error,
            page: 0,
            zoom: 1.0,
            raster: None,
            texture: None,
            log: VecDeque::new(),
            typed: String::new(),
            left_open: true,
            right_open: true,
            ui_scale: 1.0,
            frames: 0,
            last_size: egui::Vec2::ZERO,
            renderer_name,
        }
    }

    fn push_log(&mut self, line: String) {
        if self.log.len() >= LOG_LINES {
            self.log.pop_front();
        }
        self.log.push_back(line);
    }

    fn page_count(&self) -> usize {
        self.doc.as_ref().map_or(0, Doc::page_count)
    }

    fn go_to(&mut self, page: usize) {
        let last = self.page_count().saturating_sub(1);
        self.page = page.min(last);
    }

    fn zoom_step(&mut self, up: bool) {
        let next = if up {
            ZOOM_STEPS.iter().copied().find(|z| *z > self.zoom + 0.001)
        } else {
            ZOOM_STEPS.iter().rev().copied().find(|z| *z < self.zoom - 0.001)
        };
        if let Some(z) = next {
            self.zoom = z;
        }
    }

    /// Run a command: the spike's own view actions (`probe.*`, view-local in the real app too) and
    /// the engine registry's, which only phase 2 carries out (they are logged here).
    fn run(&mut self, id: &str) {
        match id {
            "probe.zoom_in" => self.zoom_step(true),
            "probe.zoom_out" => self.zoom_step(false),
            "probe.actual_size" => self.zoom = 1.0,
            "probe.next_page" => self.go_to(self.page.saturating_add(1)),
            "probe.previous_page" => self.go_to(self.page.saturating_sub(1)),
            "probe.first_page" => self.go_to(0),
            "probe.last_page" => self.go_to(usize::MAX),
            _ => self.push_log(format!("command {id} (phase 2)")),
        }
    }

    /// Record the frame's input events (pointer moves are summarised in the panel instead).
    fn record_input(&mut self, ui: &egui::Ui) {
        let events: Vec<egui::Event> = ui.input(|i| i.events.clone());
        for e in events {
            if matches!(e, egui::Event::PointerMoved(_) | egui::Event::MouseMoved(_)) {
                continue;
            }
            let line = format!("{e:?}");
            log::debug!("input: {line}");
            self.push_log(line);
        }
        let size = ui.ctx().content_rect().size();
        if size != self.last_size {
            self.last_size = size;
            let ppp = ui.ctx().pixels_per_point();
            self.push_log(format!("window {:.0}×{:.0} points, {:.2} px/pt", size.x, size.y, ppp));
        }
    }

    /// Keyboard shortcuts the spike checks (the real app gets them from the registry and craft-keymap).
    fn shortcuts(&mut self, ui: &egui::Ui) {
        use egui::{Key, KeyboardShortcut, Modifiers};
        let ctx = ui.ctx().clone();
        let pressed = |m: Modifiers, k: Key| ctx.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(m, k)));
        let text_focus = ctx.memory(|m| m.focused().is_some());
        if pressed(Modifiers::COMMAND, Key::Plus) || pressed(Modifiers::COMMAND, Key::Equals) {
            self.zoom_step(true);
        }
        if pressed(Modifiers::COMMAND, Key::Minus) {
            self.zoom_step(false);
        }
        if pressed(Modifiers::COMMAND, Key::Num0) {
            self.zoom = 1.0;
        }
        if pressed(Modifiers::COMMAND, Key::B) {
            self.left_open = !self.left_open;
        }
        if text_focus {
            return;
        }
        if pressed(Modifiers::NONE, Key::PageDown) || pressed(Modifiers::NONE, Key::ArrowRight) {
            self.go_to(self.page.saturating_add(1));
        }
        if pressed(Modifiers::NONE, Key::PageUp) || pressed(Modifiers::NONE, Key::ArrowLeft) {
            self.go_to(self.page.saturating_sub(1));
        }
        if pressed(Modifiers::NONE, Key::Home) {
            self.go_to(0);
        }
        if pressed(Modifiers::NONE, Key::End) {
            self.go_to(usize::MAX);
        }
    }

    fn menu_bar(&mut self, ui: &mut egui::Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            for menu in MENUS {
                ui.menu_button(menu, |ui| {
                    for spec in COMMANDS.iter().filter(|s| s.menu == Some(menu)) {
                        if command_button(ui, spec).clicked() {
                            self.run(spec.id);
                            ui.close();
                        }
                    }
                });
            }
        });
    }

    fn quick_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("☰").on_hover_text("All tools (Ctrl+B)").clicked() {
                self.left_open = !self.left_open;
            }
            ui.separator();
            if ui.button("⏮").on_hover_text("First page (Home)").clicked() {
                self.go_to(0);
            }
            if ui.button("◀").on_hover_text("Previous page (Page Up)").clicked() {
                self.go_to(self.page.saturating_sub(1));
            }
            let count = self.page_count();
            ui.label(format!("{} / {}", if count == 0 { 0 } else { self.page + 1 }, count));
            if ui.button("▶").on_hover_text("Next page (Page Down)").clicked() {
                self.go_to(self.page.saturating_add(1));
            }
            if ui.button("⏭").on_hover_text("Last page (End)").clicked() {
                self.go_to(usize::MAX);
            }
            ui.separator();
            if ui.button("−").on_hover_text("Zoom out (Ctrl+−)").clicked() {
                self.zoom_step(false);
            }
            ui.label(format!("{:.0}%", self.zoom * 100.0));
            if ui.button("+").on_hover_text("Zoom in (Ctrl++)").clicked() {
                self.zoom_step(true);
            }
            ui.separator();
            ui.label("Interface");
            if ui.add(egui::Slider::new(&mut self.ui_scale, 0.75..=2.0).step_by(0.05)).changed() {
                ui.ctx().set_zoom_factor(self.ui_scale);
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("ⓘ").on_hover_text("Input diagnostics").clicked() {
                    self.right_open = !self.right_open;
                }
                egui::widgets::global_theme_preference_buttons(ui);
            });
        });
    }

    /// The All tools pane, from the engine's command registry: the same groups the desktop shows.
    fn all_tools(&mut self, ui: &mut egui::Ui) {
        ui.heading("All tools");
        ui.label(format!("{} commands in the engine's registry", COMMANDS.len()));
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut groups: Vec<&str> = COMMANDS.iter().filter_map(|s| s.id.split('.').next()).collect();
            groups.dedup();
            groups.sort_unstable();
            groups.dedup();
            for group in groups {
                egui::CollapsingHeader::new(group).default_open(false).show(ui, |ui| {
                    for spec in COMMANDS.iter().filter(|s| s.id.split('.').next() == Some(group)) {
                        if command_button(ui, spec).clicked() {
                            self.run(spec.id);
                        }
                    }
                });
            }
        });
    }

    fn diagnostics(&mut self, ui: &mut egui::Ui) {
        ui.heading("Input on this device");
        let (pos, down, mods, hover) = ui.input(|i| {
            let p = &i.pointer;
            let down: Vec<&str> = [
                (egui::PointerButton::Primary, "primary"),
                (egui::PointerButton::Secondary, "secondary"),
                (egui::PointerButton::Middle, "middle"),
                (egui::PointerButton::Extra1, "back"),
                (egui::PointerButton::Extra2, "forward"),
            ]
            .into_iter()
            .filter(|(b, _)| p.button_down(*b))
            .map(|(_, n)| n)
            .collect();
            (p.hover_pos(), down.join(" + "), i.modifiers, p.has_pointer())
        });
        egui::Grid::new("pointer").num_columns(2).show(ui, |ui| {
            ui.label("Pointer");
            ui.label(pos.map_or("—".to_owned(), |p| format!("{:.0}, {:.0}", p.x, p.y)));
            ui.end_row();
            ui.label("Hovering");
            ui.label(if hover { "yes (mouse or pen)" } else { "no" });
            ui.end_row();
            ui.label("Buttons");
            ui.label(if down.is_empty() { "—" } else { &down });
            ui.end_row();
            ui.label("Modifiers");
            ui.label(format!("{mods:?}"));
            ui.end_row();
        });
        ui.add_space(4.0);
        ui.label("Type here (keyboard, IME):");
        ui.text_edit_singleline(&mut self.typed);
        ui.add_space(4.0);
        let hover_box = ui.add(egui::Button::new("Hover, right-click or middle-click me").min_size(egui::vec2(ui.available_width(), 36.0)));
        if hover_box.hovered() {
            ui.label("↑ hovered");
        }
        if hover_box.middle_clicked() {
            self.push_log("middle click on the test button".into());
        }
        hover_box.context_menu(|ui| {
            ui.label("A context menu from a right-click");
            if ui.button("Close").clicked() {
                ui.close();
            }
        });
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Events");
            if ui.small_button("Clear").clicked() {
                self.log.clear();
            }
        });
        egui::ScrollArea::vertical().stick_to_bottom(true).auto_shrink([false, false]).show(ui, |ui| {
            for line in &self.log {
                ui.add(egui::Label::new(egui::RichText::new(line).monospace().small()).wrap());
            }
        });
    }

    fn status_bar(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let ctx = ui.ctx();
            let size = ctx.content_rect().size();
            ui.label(format!(
                "{} · {} · {:.0}×{:.0} pt @ {:.2} px/pt · frame {}",
                self.platform.os,
                self.renderer_name,
                size.x,
                size.y,
                ctx.pixels_per_point(),
                self.frames
            ));
            if let Some(r) = &self.raster {
                ui.separator();
                ui.label(format!("page drawn in {} ms at {:.2} px/pt", r.millis, r.scale));
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("engine {}", pdfcraft_engine::build_info::VERSION));
            });
        });
    }

    fn page_view(&mut self, ui: &mut egui::Ui) {
        if let Some(e) = &self.error {
            ui.centered_and_justified(|ui| ui.colored_label(ui.visuals().error_fg_color, e));
            return;
        }
        let Some(doc) = self.doc.as_mut() else { return };
        let ppp = ui.ctx().pixels_per_point();
        let scale = self.zoom * ppp;
        let stale = self.raster.as_ref().is_none_or(|r| r.page != self.page || (r.scale - scale).abs() > 0.01);
        if stale {
            match doc.render(self.page, scale) {
                Ok(r) => {
                    let image = r.image.clone();
                    match &mut self.texture {
                        Some(t) => t.set(image, egui::TextureOptions::LINEAR),
                        None => self.texture = Some(ui.ctx().load_texture("page", image, egui::TextureOptions::LINEAR)),
                    }
                    self.raster = Some(r);
                }
                Err(e) => self.error = Some(e),
            }
        }
        let size = doc.page_size(self.page) * self.zoom;
        let title = format!("{} — page {}", doc.name, self.page + 1);
        let Some(texture) = &self.texture else { return };
        let mut action: Option<&'static str> = None;
        egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(12.0);
                let response = ui.add(egui::Image::new((texture.id(), size)).sense(egui::Sense::click()));
                response.clone().on_hover_text(&title);
                response.context_menu(|ui| {
                    for (label, id) in [("Zoom in", "probe.zoom_in"), ("Zoom out", "probe.zoom_out"), ("Actual size", "probe.actual_size"), ("Next page", "probe.next_page")] {
                        if ui.button(label).clicked() {
                            action = Some(id);
                            ui.close();
                        }
                    }
                });
                ui.add_space(12.0);
            });
        });
        // Ctrl+wheel and pinch: egui reports both as a zoom delta
        let zoom_delta = ui.input(|i| i.zoom_delta());
        if (zoom_delta - 1.0).abs() > 0.001 && ui.ui_contains_pointer() {
            self.zoom = (self.zoom * zoom_delta).clamp(0.1, 8.0);
        }
        if let Some(id) = action {
            self.run(id);
        }
    }
}

impl eframe::App for ProbeApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.frames = self.frames.wrapping_add(1);
        self.record_input(ui);
        self.shortcuts(ui);
        egui::Panel::top("menu_bar").show(ui, |ui| self.menu_bar(ui));
        egui::Panel::top("quick_toolbar").show(ui, |ui| self.quick_toolbar(ui));
        egui::Panel::bottom("status_bar").show(ui, |ui| self.status_bar(ui));
        if self.left_open {
            egui::Panel::left("all_tools").resizable(true).default_size(240.0).show(ui, |ui| self.all_tools(ui));
        }
        if self.right_open {
            egui::Panel::right("diagnostics").resizable(true).default_size(300.0).show(ui, |ui| self.diagnostics(ui));
        }
        egui::CentralPanel::default().show(ui, |ui| self.page_view(ui));
    }
}

fn command_button(ui: &mut egui::Ui, spec: &CommandSpec) -> egui::Response {
    let mut button = egui::Button::new(spec.label);
    if let Some(s) = spec.shortcut {
        button = button.shortcut_text(s.label(false));
    }
    ui.add(button)
}

fn renderer_name(cc: &eframe::CreationContext<'_>) -> String {
    if let Some(rs) = cc.wgpu_render_state.as_ref() {
        let info = rs.adapter.get_info();
        return format!("wgpu {:?}: {}", info.backend, info.name);
    }
    if cc.gl.is_some() {
        return "OpenGL ES (glow)".into();
    }
    "unknown renderer".into()
}
