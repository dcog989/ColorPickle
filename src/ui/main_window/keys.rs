use eframe::egui;

use crate::color::ColorFormat;
use crate::color::okhsl::Okhsl;
use crate::config::Theme;

#[derive(Clone, Copy, PartialEq)]
pub(super) struct ColorKey {
    hue: f32,
    saturation: f32,
    lightness: f32,
}

impl ColorKey {
    pub(super) fn new(color: Okhsl) -> Self {
        Self {
            hue: color.hue(),
            saturation: color.saturation(),
            lightness: color.lightness(),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(super) struct ThemeKey {
    pub(super) theme: Theme,
    pub(super) system: Option<egui::Theme>,
    pub(super) color: ColorKey,
}

#[derive(Clone, Copy, PartialEq)]
pub(super) struct InputKey {
    pub(super) format: ColorFormat,
    pub(super) color: ColorKey,
}
