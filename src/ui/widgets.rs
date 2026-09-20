use eframe::egui;

use crate::color::ColorFormat;
use crate::color::okhsl::Okhsl;
use crate::ui::theme::{color32, contrast_color32};

const PIPETTE_ICON_SIZE: f32 = 40.0;
const PIPETTE_VIEWBOX: f32 = 24.0;
const PIPETTE_STROKE: f32 = 2.0;
const COPY_ICON_SIZE: f32 = 22.0;
const COPY_ICON_STROKE_WIDTH: f32 = 1.5;
const COPY_ICON_OFFSET: f32 = 4.0;
const COPY_ICON_CORNER_RADIUS: u8 = 2;
const SWATCH_SIZE: f32 = 24.0;
const SWATCH_CORNER_RADIUS: u8 = 3;
const SWATCH_BORDER_WIDTH: f32 = 1.0;
const ROTATE_SEGMENTS: usize = 24;
const ROTATE_STROKE_WIDTH: f32 = 1.5;
const ROTATE_RADIUS_FRACTION: f32 = 0.32;
const ROTATE_ARROW_FRACTION: f32 = 0.5;
const ROTATE_SWEEP_TURNS: f32 = 1.5;

pub fn picker_launcher(ui: &mut egui::Ui, color: Okhsl) -> bool {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(PIPETTE_ICON_SIZE, PIPETTE_ICON_SIZE),
        egui::Sense::click(),
    );
    let response = response
        .on_hover_cursor(egui::CursorIcon::Crosshair)
        .on_hover_text("Pick from screen");
    paint_pipette(ui.painter(), rect, contrast_color32(color));
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

// Lucide "pipette" (https://lucide.dev/icons/pipette), paths flattened from the
// 24x24 viewBox, stroke-width 2, round caps/joins.
const PIPETTE_PATHS: [&[(f32, f32)]; 3] = [
    &[
        (12.000, 9.000),
        (3.586, 17.414),
        (3.413, 17.611),
        (3.268, 17.828),
        (3.152, 18.063),
        (3.068, 18.310),
        (3.017, 18.567),
        (3.000, 18.828),
        (3.000, 20.172),
        (2.983, 20.433),
        (2.932, 20.690),
        (2.848, 20.937),
        (2.732, 21.172),
        (2.587, 21.389),
        (2.414, 21.586),
        (2.611, 21.413),
        (2.828, 21.268),
        (3.063, 21.152),
        (3.310, 21.068),
        (3.567, 21.017),
        (3.828, 21.000),
        (5.172, 21.000),
        (5.433, 20.983),
        (5.690, 20.932),
        (5.937, 20.848),
        (6.172, 20.732),
        (6.389, 20.587),
        (6.586, 20.414),
        (15.000, 12.000),
    ],
    &[
        (18.000, 9.000),
        (18.400, 9.400),
        (18.696, 9.771),
        (18.902, 10.199),
        (19.008, 10.662),
        (19.008, 11.138),
        (18.902, 11.601),
        (18.696, 12.029),
        (18.400, 12.400),
        (18.029, 12.696),
        (17.601, 12.902),
        (17.138, 13.008),
        (16.662, 13.008),
        (16.199, 12.902),
        (15.771, 12.696),
        (15.400, 12.400),
        (11.600, 8.600),
        (11.304, 8.229),
        (11.098, 7.801),
        (10.992, 7.338),
        (10.992, 6.862),
        (11.098, 6.399),
        (11.304, 5.971),
        (11.600, 5.600),
        (11.971, 5.304),
        (12.399, 5.098),
        (12.862, 4.992),
        (13.338, 4.992),
        (13.801, 5.098),
        (14.229, 5.304),
        (14.600, 5.600),
        (15.000, 6.000),
        (18.400, 2.600),
        (18.771, 2.304),
        (19.199, 2.098),
        (19.662, 1.992),
        (20.138, 1.992),
        (20.601, 2.098),
        (21.029, 2.304),
        (21.400, 2.600),
        (21.696, 2.971),
        (21.902, 3.399),
        (22.008, 3.862),
        (22.008, 4.338),
        (21.902, 4.801),
        (21.696, 5.229),
        (21.400, 5.600),
        (18.000, 9.000),
    ],
    &[(2.000, 22.000), (2.414, 21.586)],
];

fn paint_pipette(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    let scale = rect.width() / PIPETTE_VIEWBOX;
    let stroke = egui::Stroke::new(PIPETTE_STROKE * scale, color);
    for path in PIPETTE_PATHS {
        let points = path
            .iter()
            .map(|&(x, y)| egui::pos2(rect.left() + x * scale, rect.top() + y * scale))
            .collect::<Vec<_>>();
        painter.add(egui::Shape::line(points, stroke));
    }
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
