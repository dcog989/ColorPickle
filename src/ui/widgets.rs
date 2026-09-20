use eframe::egui;

use crate::color::ColorFormat;
use crate::color::okhsl::Okhsl;
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
const GEAR_VIEWBOX: f32 = 24.0;
const GEAR_STROKE: f32 = 2.0;
const PALETTE_VIEWBOX: f32 = 24.0;
const PALETTE_STROKE: f32 = 2.0;
const PALETTE_DOT_RADIUS: f32 = 0.5;
const COPY_ICON_STROKE_WIDTH: f32 = 1.5;
const COPY_ICON_OFFSET_FRACTION: f32 = 0.18;
const COPY_ICON_CORNER_RADIUS: u8 = 2;
const SWATCH_SIZE: f32 = 24.0;
const SWATCH_CORNER_RADIUS: u8 = 6;
const SWATCH_BORDER_WIDTH: f32 = 1.0;
const CLEAR_HISTORY_PADDING: f32 = 4.0;
const BROOM_VIEWBOX: f32 = 24.0;
const BROOM_STROKE: f32 = 2.0;

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

pub fn palette_icon(ui: &mut egui::Ui, color: egui::Color32) {
    let height = row_height(ui);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(height, height), egui::Sense::hover());
    let painter = ui.painter();
    paint_paths(
        painter,
        rect,
        PALETTE_VIEWBOX,
        PALETTE_STROKE,
        color,
        &PALETTE_PATHS,
    );

    let scale = rect.width() / PALETTE_VIEWBOX;
    for &(x, y) in &PALETTE_DOTS {
        painter.circle_filled(
            egui::pos2(rect.left() + x * scale, rect.top() + y * scale),
            PALETTE_DOT_RADIUS * scale,
            color,
        );
    }
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

// Lucide "palette" (https://lucide.dev/icons/palette), outline flattened from the
// 24x24 viewBox; the paint wells are the separate filled dots below.
const PALETTE_PATHS: [&[(f32, f32)]; 1] = [&[
    (12.000, 22.000),
    (8.910, 21.500),
    (6.120, 20.100),
    (3.910, 17.900),
    (2.490, 15.100),
    (2.000, 12.000),
    (2.490, 8.910),
    (3.910, 6.120),
    (6.120, 3.910),
    (8.910, 2.490),
    (12.000, 2.000),
    (13.600, 2.110),
    (15.100, 2.440),
    (16.500, 2.980),
    (17.900, 3.720),
    (19.100, 4.640),
    (20.100, 5.710),
    (20.900, 6.910),
    (21.500, 8.220),
    (21.900, 9.590),
    (22.000, 11.000),
    (21.900, 11.800),
    (21.800, 12.500),
    (21.500, 13.300),
    (21.000, 13.900),
    (20.500, 14.500),
    (19.900, 15.000),
    (19.300, 15.500),
    (18.500, 15.800),
    (17.800, 15.900),
    (17.000, 16.000),
    (14.800, 16.000),
    (14.400, 16.000),
    (14.000, 16.200),
    (13.700, 16.400),
    (13.400, 16.600),
    (13.200, 17.000),
    (13.100, 17.300),
    (13.000, 17.700),
    (13.000, 18.100),
    (13.200, 18.500),
    (13.400, 18.800),
    (13.700, 19.200),
    (13.800, 19.500),
    (14.000, 19.900),
    (14.000, 20.300),
    (13.900, 20.700),
    (13.800, 21.000),
    (13.600, 21.400),
    (13.300, 21.600),
    (13.000, 21.800),
    (12.600, 22.000),
    (12.200, 22.000),
    (12.000, 22.000),
]];

const PALETTE_DOTS: [(f32, f32); 4] = [
    (13.5, 6.5),
    (17.5, 10.5),
    (6.5, 12.5),
    (8.5, 7.5),
];

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
