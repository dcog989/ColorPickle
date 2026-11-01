use std::path::PathBuf;

use eframe::egui;

use crate::config::{Config, LogLevel};
use crate::logging;
use crate::state::State;
use crate::ui::{ICON_PNG, WINDOW_TITLE};

const WINDOW_ID: &str = "colorpickle-about";
const LOGO_SIZE: f32 = 144.0;
const LOGO_SPACING: f32 = 12.0;
const SECTION_SPACING: f32 = 14.0;
const ROW_SPACING: f32 = 14.0;
const NAME_FONT_SIZE: f32 = 22.0;
const PATH_FONT_SIZE: f32 = 13.0;
const SECTION_TITLE_FONT_SIZE: f32 = 15.0;
const WINDOW_WIDTH: f32 = 480.0;
const LOG_ROW_LABEL: &str = "Logs";
const PATH_PADDING: f32 = 2.0;
/// egui bundles no bold font, so titles are overpainted by this much to gain weight.
const BOLD_SMUDGE: f32 = 0.5;

#[derive(Default)]
pub struct About {
    open: bool,
    logo: Option<egui::TextureHandle>,
}

impl About {
    pub fn open(&mut self) {
        self.open = true;
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    /// Returns `true` when the log level changed.
    pub fn show(&mut self, ctx: &egui::Context, config: &mut Config) -> bool {
        if !self.open {
            return false;
        }
        if self.logo.is_none() {
            self.logo = logo_texture(ctx);
        }

        let mut open = self.open;
        let mut changed = false;
        egui::Window::new(format!("About {WINDOW_TITLE}"))
            .id(egui::Id::unique(WINDOW_ID))
            .collapsible(false)
            .resizable(false)
            .default_width(WINDOW_WIDTH)
            .open(&mut open)
            .show(ctx, |ui| changed = self.contents(ui, &mut config.log_level));
        self.open = open;
        changed
    }

    fn contents(&self, ui: &mut egui::Ui, level: &mut LogLevel) -> bool {
        ui.vertical_centered(|ui| {
            if let Some(logo) = &self.logo {
                ui.add(egui::Image::new(logo).fit_to_exact_size(egui::vec2(LOGO_SIZE, LOGO_SIZE)));
                ui.add_space(LOGO_SPACING);
            }
            ui.label(
                egui::RichText::new(WINDOW_TITLE)
                    .size(NAME_FONT_SIZE)
                    .strong(),
            );
            ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
        });

        ui.add_space(SECTION_SPACING);
        ui.separator();
        ui.add_space(SECTION_SPACING);

        let mut changed = false;
        for (label, path) in paths() {
            section_title(ui, label);
            path_value(ui, path);
            if label == LOG_ROW_LABEL {
                changed = log_level_combo(ui, level);
            }
            ui.add_space(ROW_SPACING);
        }
        changed
    }
}

fn section_title(ui: &mut egui::Ui, text: &str) {
    let font = egui::FontId::proportional(SECTION_TITLE_FONT_SIZE);
    let color = ui.visuals().strong_text_color();
    let galley = ui.painter().layout_no_wrap(text.to_owned(), font, color);
    let (rect, _) = ui.allocate_exact_size(galley.size(), egui::Sense::hover());
    let painter = ui.painter();
    painter.galley(rect.left_top(), galley.clone(), color);
    painter.galley(
        rect.left_top() + egui::vec2(BOLD_SMUDGE, 0.0),
        galley,
        color,
    );
}

fn log_level_combo(ui: &mut egui::Ui, level: &mut LogLevel) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label("Log level");
        egui::ComboBox::from_id_salt("log-level")
            .selected_text(level.as_str())
            .show_ui(ui, |ui| {
                for candidate in LogLevel::ALL {
                    if ui
                        .selectable_value(level, candidate, candidate.as_str())
                        .changed()
                    {
                        changed = true;
                    }
                }
            })
            .response
            .on_hover_text("Log level: applies immediately and saves to the config file.");
    });
    changed
}

fn paths() -> [(&'static str, Option<PathBuf>); 4] {
    [
        ("Config", Config::path()),
        ("History", State::path()),
        (LOG_ROW_LABEL, logging::directory()),
        ("Binary", std::env::current_exe().ok()),
    ]
}

fn path_value(ui: &mut egui::Ui, path: Option<PathBuf>) {
    let Some(path) = path else {
        ui.weak("unavailable");
        return;
    };

    let text = path.display().to_string();
    let font = egui::FontId::monospace(PATH_FONT_SIZE);
    let color = ui.visuals().text_color();
    let mut job = egui::text::LayoutJob::simple(text.clone(), font, color, f32::INFINITY);
    job.wrap =
        egui::text::TextWrapping::truncate_at_width(ui.available_width() - 2.0 * PATH_PADDING);
    let galley = ui.painter().layout_job(job);

    // The row is hard to hit at glyph height, so pad the hit area vertically.
    let size = galley.size() + egui::vec2(2.0, 2.0) * PATH_PADDING;
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    let response = response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text("Click to copy path.");
    if response.hovered() {
        let visuals = ui.style().interact(&response);
        ui.painter()
            .rect_filled(rect, visuals.corner_radius, visuals.weak_bg_fill);
        ui.painter().rect_stroke(
            rect,
            visuals.corner_radius,
            visuals.bg_stroke,
            egui::StrokeKind::Inside,
        );
    }
    ui.painter().galley(
        rect.left_top() + egui::vec2(PATH_PADDING, PATH_PADDING),
        galley,
        color,
    );
    if response.clicked() {
        ui.ctx().copy_text(text);
    }
}

fn logo_texture(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let image = image::load_from_memory(ICON_PNG).ok()?.into_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &image.into_raw());
    Some(ctx.load_texture(
        "colorpickle-logo",
        color_image,
        egui::TextureOptions::LINEAR,
    ))
}
