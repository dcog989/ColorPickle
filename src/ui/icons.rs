use std::sync::Arc;

use eframe::egui;
use lucide_icons::Icon;

const FONT_FAMILY: &str = "lucide";

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

pub fn pipette(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    glyph(painter, rect, color, Icon::Pipette);
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

