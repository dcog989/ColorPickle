use eframe::egui;

use crate::color::ColorFormat;
use crate::color::okhsl::Okhsl;
use crate::ui::icons;
use crate::ui::theme::color32;

const LOGO_PADDING: f32 = 8.0;
const LOGO_VIEWBOX: f32 = 512.0;
const LOGO_RADIUS: f32 = 256.0;
const LOGO_CIRCLE_STROKE: f32 = 24.0;
const LOGO_TINE_X: [f32; 4] = [186.0, 226.0, 266.0, 306.0];
const LOGO_TINE_WIDTH: f32 = 20.0;
const LOGO_TINE_TOP: f32 = 100.0;
const LOGO_TINE_BOTTOM: f32 = 210.0;
const LOGO_TINE_RADIUS: f32 = 10.0;
const LOGO_SHOULDER_LEFT: f32 = 186.0;
const LOGO_SHOULDER_RIGHT: f32 = 326.0;
const LOGO_SHOULDER_TOP: f32 = 190.0;
const LOGO_SHOULDER_BOTTOM: f32 = 258.0;
const LOGO_HANDLE_LEFT: f32 = 239.0;
const LOGO_HANDLE_RIGHT: f32 = 273.0;
const LOGO_HANDLE_BOTTOM: f32 = 412.0;
const LOGO_HANDLE_RADIUS: f32 = 17.0;
const COPY_ICON_STROKE_WIDTH: f32 = 1.5;
const COPY_ICON_OFFSET_FRACTION: f32 = 0.18;
const COPY_ICON_CORNER_RADIUS: u8 = 2;
const SWATCH_SIZE: f32 = 24.0;
const SWATCH_CORNER_RADIUS: u8 = 6;
const SWATCH_BORDER_WIDTH: f32 = 1.0;
const CLEAR_HISTORY_PADDING: f32 = 4.0;

pub fn picker_launcher(ui: &mut egui::Ui, height: f32) -> bool {
    icon_button(
        ui,
        egui::vec2(height, height),
        "Launch screen picker",
        LOGO_PADDING,
        paint_logo,
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
    let painter = ui.painter();
    let stroke = egui::Stroke::new(COPY_ICON_STROKE_WIDTH, color);
    let corner = egui::CornerRadius::same(COPY_ICON_CORNER_RADIUS);
    let offset = rect.width() * COPY_ICON_OFFSET_FRACTION;
    let size = egui::vec2(rect.width() - offset, rect.height() - offset);
    let front = egui::Rect::from_min_size(rect.min, size);
    let back = front.translate(egui::vec2(offset, offset));
    painter.rect_stroke(back, corner, stroke, egui::StrokeKind::Inside);
    painter.rect_stroke(front, corner, stroke, egui::StrokeKind::Inside);
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
        icons::broom,
    )
}

// Logo mark: the same fork-in-circle geometry as packaging/colorpickle.svg,
// flattened from its 512x512 viewBox. Painted in the current foreground colour
// so the launcher follows the dynamic theme like the other icons.
fn paint_logo(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    let scale = rect.width() / LOGO_VIEWBOX;
    let map = |x: f32, y: f32| egui::pos2(rect.left() + x * scale, rect.top() + y * scale);

    painter.circle_stroke(
        map(LOGO_RADIUS, LOGO_RADIUS),
        (LOGO_RADIUS - LOGO_CIRCLE_STROKE / 2.0) * scale,
        egui::Stroke::new(LOGO_CIRCLE_STROKE * scale, color),
    );

    let handle_radius = (LOGO_HANDLE_RADIUS * scale).round() as u8;
    painter.rect_filled(
        egui::Rect::from_min_max(
            map(LOGO_HANDLE_LEFT, LOGO_SHOULDER_BOTTOM),
            map(LOGO_HANDLE_RIGHT, LOGO_HANDLE_BOTTOM),
        ),
        egui::CornerRadius {
            nw: 0,
            ne: 0,
            sw: handle_radius,
            se: handle_radius,
        },
        color,
    );

    painter.add(egui::Shape::convex_polygon(
        vec![
            map(LOGO_SHOULDER_LEFT, LOGO_SHOULDER_TOP),
            map(LOGO_SHOULDER_RIGHT, LOGO_SHOULDER_TOP),
            map(LOGO_HANDLE_RIGHT, LOGO_SHOULDER_BOTTOM),
            map(LOGO_HANDLE_LEFT, LOGO_SHOULDER_BOTTOM),
        ],
        color,
        egui::Stroke::NONE,
    ));

    let tine_radius = (LOGO_TINE_RADIUS * scale).round() as u8;
    for x in LOGO_TINE_X {
        painter.rect_filled(
            egui::Rect::from_min_max(
                map(x, LOGO_TINE_TOP),
                map(x + LOGO_TINE_WIDTH, LOGO_TINE_BOTTOM),
            ),
            egui::CornerRadius {
                nw: tine_radius,
                ne: tine_radius,
                sw: 0,
                se: 0,
            },
            color,
        );
    }
}
