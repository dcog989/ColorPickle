use eframe::egui;

use crate::color::ColorFormat;
use crate::color::okhsl::Okhsl;
use crate::ui::theme::{color32, contrast_color32};

const EYEDROP_ICON_SIZE: f32 = 22.0;
const EYEDROP_OUTER_RADIUS: f32 = 8.0;
const EYEDROP_INNER_RADIUS: f32 = 2.5;
const EYEDROP_STROKE_WIDTH: f32 = 1.5;
const COPY_ICON_SIZE: f32 = 18.0;
const COPY_ICON_STROKE_WIDTH: f32 = 1.5;
const COPY_ICON_OFFSET: f32 = 3.0;
const COPY_ICON_CORNER_RADIUS: u8 = 2;
const SWATCH_SIZE: f32 = 18.0;
const SWATCH_CORNER_RADIUS: u8 = 3;
const SWATCH_BORDER_WIDTH: f32 = 1.0;
const ROTATE_SEGMENTS: usize = 24;
const ROTATE_STROKE_WIDTH: f32 = 1.5;
const ROTATE_RADIUS_FRACTION: f32 = 0.32;
const ROTATE_ARROW_FRACTION: f32 = 0.5;
const ROTATE_SWEEP_TURNS: f32 = 1.5;

pub fn picker_launcher(ui: &mut egui::Ui, color: Okhsl) -> bool {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(EYEDROP_ICON_SIZE, EYEDROP_ICON_SIZE),
        egui::Sense::click(),
    );
    let response = response
        .on_hover_cursor(egui::CursorIcon::Crosshair)
        .on_hover_text("Pick from screen");
    paint_crosshair(ui.painter(), rect, contrast_color32(color));
    response.clicked()
}

pub fn copy_icon(ui: &mut egui::Ui, color: egui::Color32) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(COPY_ICON_SIZE, COPY_ICON_SIZE),
        egui::Sense::hover(),
    );
    let painter = ui.painter();
    let stroke = egui::Stroke::new(COPY_ICON_STROKE_WIDTH, color);
    let corner = egui::CornerRadius::same(COPY_ICON_CORNER_RADIUS);
    let size = egui::vec2(
        rect.width() - COPY_ICON_OFFSET,
        rect.height() - COPY_ICON_OFFSET,
    );
    let front = egui::Rect::from_min_size(rect.min, size);
    let back = front.translate(egui::vec2(COPY_ICON_OFFSET, COPY_ICON_OFFSET));
    painter.rect_stroke(back, corner, stroke, egui::StrokeKind::Inside);
    painter.rect_stroke(front, corner, stroke, egui::StrokeKind::Inside);
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
        .on_hover_text(ColorFormat::Hex.format(color))
        .clicked()
}

pub fn clear_history(ui: &mut egui::Ui, color: egui::Color32) -> bool {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(SWATCH_SIZE, SWATCH_SIZE), egui::Sense::click());
    let response = response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text("Clear history");
    paint_rotate_ccw(ui.painter(), rect, color);
    response.clicked()
}

fn paint_crosshair(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    let center = rect.center();
    let stroke = egui::Stroke::new(EYEDROP_STROKE_WIDTH, color);
    painter.circle_stroke(center, EYEDROP_OUTER_RADIUS, stroke);
    painter.circle_filled(center, EYEDROP_INNER_RADIUS, color);
    painter.line_segment(
        [
            egui::pos2(center.x - EYEDROP_OUTER_RADIUS, center.y),
            egui::pos2(center.x + EYEDROP_OUTER_RADIUS, center.y),
        ],
        stroke,
    );
    painter.line_segment(
        [
            egui::pos2(center.x, center.y - EYEDROP_OUTER_RADIUS),
            egui::pos2(center.x, center.y + EYEDROP_OUTER_RADIUS),
        ],
        stroke,
    );
}

fn paint_rotate_ccw(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    let center = rect.center();
    let radius = rect.width() * ROTATE_RADIUS_FRACTION;
    let start = -std::f32::consts::FRAC_PI_2;
    let sweep = ROTATE_SWEEP_TURNS * std::f32::consts::TAU;

    let mut points = Vec::with_capacity(ROTATE_SEGMENTS + 1);
    for step in 0..=ROTATE_SEGMENTS {
        let angle = start + sweep * (step as f32 / ROTATE_SEGMENTS as f32);
        points.push(egui::pos2(
            center.x + radius * angle.cos(),
            center.y + radius * angle.sin(),
        ));
    }

    let tip = points[0];
    let next = points[1];
    let direction = (tip - next).normalized();
    let perpendicular = egui::vec2(-direction.y, direction.x);
    let head = radius * ROTATE_ARROW_FRACTION;

    painter.add(egui::Shape::line(
        points,
        egui::Stroke::new(ROTATE_STROKE_WIDTH, color),
    ));
    painter.add(egui::Shape::convex_polygon(
        vec![
            tip + direction * head,
            tip + perpendicular * head * 0.6,
            tip - perpendicular * head * 0.6,
        ],
        color,
        egui::Stroke::NONE,
    ));
}
