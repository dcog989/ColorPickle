use eframe::egui;

use crate::color::ColorFormat;
use crate::color::okhsl::Okhsl;
use crate::ui::icons;
use crate::ui::theme::color32;

const LAUNCHER_PADDING: f32 = 8.0;
const SWATCH_SIZE: f32 = 24.0;
const SWATCH_CORNER_RADIUS: u8 = 6;
const SWATCH_BORDER_WIDTH: f32 = 1.0;
const CLEAR_HISTORY_PADDING: f32 = 4.0;

pub fn picker_launcher(ui: &mut egui::Ui, height: f32) -> bool {
    icon_button(
        ui,
        egui::vec2(height, height),
        "Launch screen picker",
        LAUNCHER_PADDING,
        icons::pipette,
    )
}

fn icon_button(
    ui: &mut egui::Ui,
    size: egui::Vec2,
    tooltip: &str,
    padding: f32,
    draw_icon: impl FnOnce(&egui::Painter, egui::Rect, egui::Color32),
) -> bool {
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    let response = response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(tooltip);

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let button_rect = rect.expand(visuals.expansion);
        let painter = ui.painter();
        painter.rect_filled(button_rect, visuals.corner_radius, visuals.weak_bg_fill);
        painter.rect_stroke(
            button_rect,
            visuals.corner_radius,
            visuals.bg_stroke,
            egui::StrokeKind::Inside,
        );
        draw_icon(painter, rect.shrink(padding), visuals.fg_stroke.color);
    }

    response.clicked()
}

pub fn settings_icon(ui: &mut egui::Ui, color: egui::Color32) {
    let rect = icon_slot(ui);
    icons::settings(ui.painter(), rect, color);
}

pub fn palette_icon(ui: &mut egui::Ui, color: egui::Color32) {
    let rect = icon_slot(ui);
    icons::palette(ui.painter(), rect, color);
}

pub fn copy_icon(ui: &mut egui::Ui, color: egui::Color32) {
    let rect = icon_slot(ui);
    icons::copy(ui.painter(), rect, color);
}

fn icon_slot(ui: &mut egui::Ui) -> egui::Rect {
    let height = row_height(ui);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(height, height), egui::Sense::hover());
    rect
}

pub fn row_height(ui: &egui::Ui) -> f32 {
    let text = ui.text_style_height(&egui::TextStyle::Button);
    (text + 2.0 * ui.spacing().button_padding.y).max(ui.spacing().interact_size.y)
}

pub fn history_swatch(ui: &mut egui::Ui, color: Okhsl, border: egui::Color32) -> bool {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(SWATCH_SIZE, SWATCH_SIZE), egui::Sense::click());
    let corner = egui::CornerRadius::same(SWATCH_CORNER_RADIUS);
    ui.painter().rect_filled(rect, corner, color32(color));
    ui.painter().rect_stroke(
        rect,
        corner,
        egui::Stroke::new(SWATCH_BORDER_WIDTH, border),
        egui::StrokeKind::Inside,
    );
    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_ui(|ui| {
            ui.label(ColorFormat::Hex.format(color));
        })
        .clicked()
}

pub fn clear_history(ui: &mut egui::Ui) -> bool {
    let height = row_height(ui);
    icon_button(
        ui,
        egui::vec2(height, height),
        "Clear history",
        CLEAR_HISTORY_PADDING,
        icons::close,
    )
}
