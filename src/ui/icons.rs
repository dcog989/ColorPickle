use std::sync::Arc;

use eframe::egui;
use lucide_icons::Icon;

const FONT_FAMILY: &str = "lucide";

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

pub fn install(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        FONT_FAMILY.to_owned(),
        Arc::new(egui::FontData::from_static(lucide_icons::LUCIDE_FONT_BYTES)),
    );
    fonts.families.insert(
        egui::FontFamily::Name(FONT_FAMILY.into()),
        vec![FONT_FAMILY.to_owned()],
    );
    ctx.set_fonts(fonts);
}

pub fn settings(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    glyph(painter, rect, color, Icon::Settings);
}

pub fn palette(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    glyph(painter, rect, color, Icon::Palette);
}

pub fn close(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    glyph(painter, rect, color, Icon::X);
}

pub fn copy(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    glyph(painter, rect, color, Icon::Copy);
}

fn glyph(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32, icon: Icon) {
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        char::from(icon),
        egui::FontId::new(rect.height(), egui::FontFamily::Name(FONT_FAMILY.into())),
        color,
    );
}

// Logo mark: the fork-in-circle geometry from packaging/colorpickle.svg. The ring
// is stroked and the fork is filled in the current foreground color so the
// launcher follows the dynamic theme like the other icons.
pub fn logo(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
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
