use eframe::egui;
use palette::Srgb;

use crate::color::ColorFormat;
use crate::color::apca;
use crate::color::okhsl::Okhsl;
use crate::ui::icons;
use crate::ui::theme::{color32, contrast_color32};

const LAUNCHER_PADDING: f32 = 8.0;
const SWATCH_SIZE: f32 = 24.0;
const SWATCH_CORNER_RADIUS: u8 = 6;
const HISTORY_SWATCH_SIZE: f32 = SWATCH_SIZE * 2.0;
const HISTORY_SWATCH_CORNER_RADIUS: u8 = SWATCH_CORNER_RADIUS * 2;
const SWATCH_BORDER_WIDTH: f32 = 1.0;
const CLOSE_MARK_FRACTION: f32 = 0.35;
const CLOSE_MARK_MIN: f32 = 10.0;
const CLOSE_MARK_MARGIN: f32 = 6.0;
const CLOSE_MARK_PADDING: f32 = 2.0;
const CLOSE_MARK_CORNER_RADIUS: u8 = 3;
const CLOSE_MARK_FILL_ALPHA: u8 = 40;
const CLEAR_HISTORY_PADDING: f32 = 4.0;
const ABOUT_PADDING: f32 = 4.0;
const APCA_FONT_SIZE: f32 = 14.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SwatchAction {
    Select,
    Remove,
}

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

pub fn about_icon(ui: &mut egui::Ui) -> bool {
    icon_button(
        ui,
        egui::vec2(row_height(ui), row_height(ui)),
        "About ColorPickle",
        ABOUT_PADDING,
        icons::info,
    )
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

pub fn history_swatch(
    ui: &mut egui::Ui,
    color: Okhsl,
    border: egui::Color32,
    format: ColorFormat,
) -> Option<SwatchAction> {
    swatch(
        ui,
        color,
        border,
        format,
        HISTORY_SWATCH_SIZE,
        HISTORY_SWATCH_CORNER_RADIUS,
        true,
    )
}

pub fn harmony_swatch(
    ui: &mut egui::Ui,
    color: Okhsl,
    border: egui::Color32,
    format: ColorFormat,
) -> bool {
    swatch(
        ui,
        color,
        border,
        format,
        SWATCH_SIZE,
        SWATCH_CORNER_RADIUS,
        false,
    ) == Some(SwatchAction::Select)
}

fn swatch(
    ui: &mut egui::Ui,
    color: Okhsl,
    border: egui::Color32,
    format: ColorFormat,
    size: f32,
    corner_radius: u8,
    closable: bool,
) -> Option<SwatchAction> {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::click());
    let corner = egui::CornerRadius::same(corner_radius);
    ui.painter().rect_filled(rect, corner, color32(color));
    ui.painter().rect_stroke(
        rect,
        corner,
        egui::Stroke::new(SWATCH_BORDER_WIDTH, border),
        egui::StrokeKind::Inside,
    );

    let response = response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_ui(|ui| {
            ui.label(format.format(color));
        });

    if closable && ui.rect_contains_pointer(rect) {
        let mark = close_mark_rect(rect);
        let close = ui
            .interact(mark, response.id.with("close"), egui::Sense::click())
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        draw_close_mark(ui.painter(), mark, color);
        if close.clicked() {
            return Some(SwatchAction::Remove);
        }
    }

    response.clicked().then_some(SwatchAction::Select)
}

fn close_mark_rect(rect: egui::Rect) -> egui::Rect {
    let size = (rect.width() * CLOSE_MARK_FRACTION).max(CLOSE_MARK_MIN);
    egui::Rect::from_min_size(
        egui::pos2(
            rect.right() - CLOSE_MARK_MARGIN - size,
            rect.top() + CLOSE_MARK_MARGIN,
        ),
        egui::vec2(size, size),
    )
}

fn draw_close_mark(painter: &egui::Painter, mark: egui::Rect, color: Okhsl) {
    let contrast = contrast_color32(color);
    let corner = egui::CornerRadius::same(CLOSE_MARK_CORNER_RADIUS);
    let fill = egui::Color32::from_rgba_unmultiplied(
        contrast.r(),
        contrast.g(),
        contrast.b(),
        CLOSE_MARK_FILL_ALPHA,
    );
    painter.rect_filled(mark, corner, fill);
    painter.rect_stroke(
        mark,
        corner,
        egui::Stroke::new(SWATCH_BORDER_WIDTH, contrast),
        egui::StrokeKind::Inside,
    );
    icons::close(painter, mark.shrink(CLOSE_MARK_PADDING), contrast);
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

pub fn apca_scores(ui: &mut egui::Ui, color: Okhsl, foreground: egui::Color32) {
    let srgb = color.to_srgb();
    let on_white = apca::contrast(srgb, Srgb::new(1.0, 1.0, 1.0)).abs();
    let on_black = apca::contrast(srgb, Srgb::new(0.0, 0.0, 0.0)).abs();
    let text = format!("APCA score: {on_white:.0} (white), {on_black:.0} (black)");
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        ui.label(
            egui::RichText::new(text)
                .color(foreground)
                .size(APCA_FONT_SIZE),
        )
        .on_hover_text("APCA lightness contrast (Lc) of this colour on white and on black");
    });
}
