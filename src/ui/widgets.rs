use eframe::egui;

use crate::color::ColorFormat;
use crate::color::okhsl::Okhsl;
use crate::ui::theme::color32;

const PIPETTE_PADDING: f32 = 8.0;
const PIPETTE_VIEWBOX: f32 = 24.0;
const PIPETTE_STROKE: f32 = 2.0;
const GEAR_VIEWBOX: f32 = 24.0;
const GEAR_STROKE: f32 = 2.0;
const COPY_ICON_STROKE_WIDTH: f32 = 1.5;
const COPY_ICON_OFFSET_FRACTION: f32 = 0.18;
const COPY_ICON_CORNER_RADIUS: u8 = 2;
const SWATCH_SIZE: f32 = 24.0;
const SWATCH_CORNER_RADIUS: u8 = 3;
const SWATCH_BORDER_WIDTH: f32 = 1.0;
const CLEAR_HISTORY_PADDING: f32 = 4.0;
const BROOM_VIEWBOX: f32 = 24.0;
const BROOM_STROKE: f32 = 2.0;

pub fn picker_launcher(ui: &mut egui::Ui, height: f32) -> bool {
    icon_button(
        ui,
        egui::vec2(height, height),
        "Launch screen picker",
        PIPETTE_PADDING,
        paint_pipette,
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
    let height = row_height(ui);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(height, height), egui::Sense::hover());
    paint_paths(
        ui.painter(),
        rect,
        GEAR_VIEWBOX,
        GEAR_STROKE,
        color,
        &GEAR_PATHS,
    );
}

pub fn copy_icon(ui: &mut egui::Ui, color: egui::Color32) {
    let height = row_height(ui);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(height, height), egui::Sense::hover());
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
        .on_hover_text(ColorFormat::Hex.format(color))
        .clicked()
}

pub fn clear_history(ui: &mut egui::Ui) -> bool {
    let height = row_height(ui);
    icon_button(
        ui,
        egui::vec2(height, height),
        "Clear history",
        CLEAR_HISTORY_PADDING,
        paint_broom,
    )
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

// Lucide "settings" (https://lucide.dev/icons/settings): the cog outline plus the
// centre hole, both flattened from the 24x24 viewBox.
const GEAR_PATHS: [&[(f32, f32)]; 2] = [
    &[
        (9.671, 4.136),
        (9.813, 3.526),
        (10.112, 2.976),
        (10.546, 2.525),
        (11.084, 2.204),
        (11.687, 2.038),
        (12.314, 2.038),
        (12.917, 2.204),
        (13.455, 2.525),
        (13.889, 2.976),
        (14.188, 3.526),
        (14.330, 4.136),
        (14.473, 4.759),
        (14.779, 5.320),
        (15.226, 5.778),
        (15.779, 6.097),
        (16.399, 6.255),
        (17.038, 6.239),
        (17.649, 6.051),
        (18.247, 5.871),
        (18.872, 5.856),
        (19.478, 6.006),
        (20.023, 6.312),
        (20.468, 6.751),
        (20.781, 7.292),
        (20.939, 7.897),
        (20.932, 8.521),
        (20.759, 9.122),
        (20.434, 9.656),
        (19.979, 10.084),
        (19.511, 10.519),
        (19.178, 11.065),
        (19.005, 11.680),
        (19.005, 12.319),
        (19.178, 12.934),
        (19.511, 13.480),
        (19.979, 13.915),
        (20.434, 14.343),
        (20.759, 14.877),
        (20.932, 15.478),
        (20.939, 16.102),
        (20.781, 16.707),
        (20.468, 17.248),
        (20.023, 17.687),
        (19.478, 17.993),
        (18.872, 18.143),
        (18.247, 18.128),
        (17.649, 17.948),
        (17.038, 17.760),
        (16.399, 17.744),
        (15.779, 17.902),
        (15.226, 18.221),
        (14.779, 18.679),
        (14.473, 19.240),
        (14.330, 19.863),
        (14.188, 20.473),
        (13.889, 21.023),
        (13.455, 21.474),
        (12.917, 21.795),
        (12.314, 21.961),
        (11.687, 21.961),
        (11.084, 21.795),
        (10.546, 21.474),
        (10.112, 21.023),
        (9.813, 20.473),
        (9.671, 19.863),
        (9.528, 19.240),
        (9.222, 18.678),
        (8.775, 18.221),
        (8.221, 17.901),
        (7.601, 17.743),
        (6.962, 17.759),
        (6.351, 17.948),
        (5.753, 18.128),
        (5.128, 18.143),
        (4.522, 17.993),
        (3.977, 17.687),
        (3.532, 17.248),
        (3.219, 16.707),
        (3.061, 16.102),
        (3.068, 15.478),
        (3.241, 14.877),
        (3.566, 14.343),
        (4.021, 13.915),
        (4.489, 13.480),
        (4.822, 12.934),
        (4.995, 12.319),
        (4.995, 11.680),
        (4.822, 11.065),
        (4.489, 10.519),
        (4.021, 10.084),
        (3.567, 9.656),
        (3.242, 9.122),
        (3.070, 8.522),
        (3.063, 7.898),
        (3.221, 7.294),
        (3.533, 6.753),
        (3.977, 6.314),
        (4.522, 6.008),
        (5.128, 5.857),
        (5.752, 5.872),
        (6.350, 6.051),
        (6.961, 6.239),
        (7.600, 6.255),
        (8.220, 6.097),
        (8.773, 5.778),
        (9.220, 5.320),
        (9.526, 4.759),
        (9.669, 4.136),
    ],
    &[
        (15.000, 12.000),
        (14.963, 12.469),
        (14.853, 12.927),
        (14.673, 13.362),
        (14.427, 13.763),
        (14.121, 14.121),
        (13.763, 14.427),
        (13.362, 14.673),
        (12.927, 14.853),
        (12.469, 14.963),
        (12.000, 15.000),
        (11.531, 14.963),
        (11.073, 14.853),
        (10.638, 14.673),
        (10.237, 14.427),
        (9.879, 14.121),
        (9.573, 13.763),
        (9.327, 13.362),
        (9.147, 12.927),
        (9.037, 12.469),
        (9.000, 12.000),
        (9.037, 11.531),
        (9.147, 11.073),
        (9.327, 10.638),
        (9.573, 10.237),
        (9.879, 9.879),
        (10.237, 9.573),
        (10.638, 9.327),
        (11.073, 9.147),
        (11.531, 9.037),
        (12.000, 9.000),
        (12.469, 9.037),
        (12.927, 9.147),
        (13.362, 9.327),
        (13.763, 9.573),
        (14.121, 9.879),
        (14.427, 10.237),
        (14.673, 10.638),
        (14.853, 11.073),
        (14.963, 11.531),
        (15.000, 12.000),
    ],
];

fn paint_pipette(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    paint_paths(
        painter,
        rect,
        PIPETTE_VIEWBOX,
        PIPETTE_STROKE,
        color,
        &PIPETTE_PATHS,
    );
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

// Lucide "broom" (https://lucide.dev/icons/broom), flattened from the 24x24 viewBox.
const BROOM_PATHS: [&[(f32, f32)]; 4] = [
    &[(13.500, 10.500), (22.000, 2.000)],
    &[
        (14.734, 13.841),
        (14.936, 13.358),
        (15.005, 12.839),
        (14.938, 12.320),
        (14.738, 11.836),
        (14.420, 11.421),
        (12.580, 9.580),
        (12.164, 9.261),
        (11.680, 9.062),
        (11.161, 8.994),
        (10.642, 9.064),
        (10.159, 9.266),
        (2.502, 13.727),
        (2.266, 13.918),
        (2.098, 14.172),
        (2.013, 14.464),
        (2.021, 14.768),
        (2.120, 15.055),
        (2.300, 15.300),
        (8.703, 21.703),
        (8.948, 21.883),
        (9.235, 21.980),
        (9.539, 21.987),
        (9.830, 21.903),
        (10.083, 21.735),
        (10.274, 21.499),
        (14.734, 13.841),
    ],
    &[(5.000, 18.000), (7.000, 16.000)],
    &[(7.699, 10.700), (13.301, 16.301)],
];

fn paint_broom(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    paint_paths(
        painter,
        rect,
        BROOM_VIEWBOX,
        BROOM_STROKE,
        color,
        &BROOM_PATHS,
    );
}
