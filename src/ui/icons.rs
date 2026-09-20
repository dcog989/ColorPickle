use std::f32::consts::TAU;

use eframe::egui;

const VIEWBOX: f32 = 24.0;
const STROKE: f32 = 2.0;
const PALETTE_DOT_RADIUS: f32 = 0.9;
const PALETTE_DOTS: [(f32, f32); 4] = [(10.0, 9.0), (14.5, 9.5), (15.0, 13.5), (10.5, 13.5)];

pub fn settings(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    let paths = [gear(), circle(12.0, 12.0, 3.2)];
    paint(painter, rect, color, &paths);
}

pub fn palette(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    let paths = [ellipse(12.0, 12.0, 9.0, 7.2), circle(6.5, 14.5, 1.7)];
    paint(painter, rect, color, &paths);

    let scale = rect.width() / VIEWBOX;
    for &(x, y) in &PALETTE_DOTS {
        painter.circle_filled(
            egui::pos2(rect.left() + x * scale, rect.top() + y * scale),
            PALETTE_DOT_RADIUS * scale,
            color,
        );
    }
}

pub fn broom(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    let paths = [
        vec![(20.0, 4.0), (11.5, 12.5)],
        vec![
            (11.5, 12.5),
            (6.0, 18.0),
            (9.0, 21.0),
            (14.5, 15.5),
            (11.5, 12.5),
        ],
        vec![(6.0, 18.0), (4.0, 20.0)],
        vec![(7.5, 19.5), (6.0, 21.0)],
        vec![(9.0, 21.0), (7.5, 22.5)],
    ];
    paint(painter, rect, color, &paths);
}

fn paint(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32, paths: &[Vec<(f32, f32)>]) {
    let paths: Vec<&[(f32, f32)]> = paths.iter().map(|path| path.as_slice()).collect();
    paint_paths(painter, rect, VIEWBOX, STROKE, color, &paths);
}

fn gear() -> Vec<(f32, f32)> {
    const TEETH: usize = 8;
    const OUTER: f32 = 9.5;
    const INNER: f32 = 7.0;
    let step = TAU / TEETH as f32;
    let tooth = step * 0.18;
    let slope = step * 0.10;

    let mut points = Vec::with_capacity(TEETH * 4 + 1);
    for index in 0..TEETH {
        let base = index as f32 * step;
        points.push(polar(base - tooth, OUTER));
        points.push(polar(base + tooth, OUTER));
        points.push(polar(base + tooth + slope, INNER));
        points.push(polar(base + step - tooth - slope, INNER));
    }
    points.push(points[0]);
    points
}

fn circle(center_x: f32, center_y: f32, radius: f32) -> Vec<(f32, f32)> {
    const SEGMENTS: usize = 32;
    let mut points: Vec<(f32, f32)> = (0..SEGMENTS)
        .map(|index| {
            let angle = index as f32 / SEGMENTS as f32 * TAU;
            (
                center_x + radius * angle.cos(),
                center_y + radius * angle.sin(),
            )
        })
        .collect();
    points.push(points[0]);
    points
}

fn ellipse(center_x: f32, center_y: f32, radius_x: f32, radius_y: f32) -> Vec<(f32, f32)> {
    const SEGMENTS: usize = 40;
    let mut points: Vec<(f32, f32)> = (0..SEGMENTS)
        .map(|index| {
            let angle = index as f32 / SEGMENTS as f32 * TAU;
            (
                center_x + radius_x * angle.cos(),
                center_y + radius_y * angle.sin(),
            )
        })
        .collect();
    points.push(points[0]);
    points
}

fn polar(angle: f32, radius: f32) -> (f32, f32) {
    (12.0 + radius * angle.cos(), 12.0 + radius * angle.sin())
}

fn paint_paths(
    painter: &egui::Painter,
    rect: egui::Rect,
    viewbox: f32,
    stroke_width: f32,
    color: egui::Color32,
    paths: &[&[(f32, f32)]],
) {
    let scale = rect.width() / viewbox;
    let stroke = egui::Stroke::new(stroke_width * scale, color);
    for path in paths {
        let points = path
            .iter()
            .map(|&(x, y)| egui::pos2(rect.left() + x * scale, rect.top() + y * scale))
            .collect::<Vec<_>>();
        painter.add(egui::Shape::line(points, stroke));
    }
}
