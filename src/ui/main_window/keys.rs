use eframe::egui;

use crate::color::ColorFormat;
use crate::color::okhsl::Okhsl;
use crate::config::Theme;

#[derive(Clone, Copy, PartialEq)]
pub(super) struct ThemeKey {
    pub(super) theme: Theme,
    pub(super) system: Option<egui::Theme>,
    pub(super) color: Okhsl,
}

#[derive(Clone, Copy, PartialEq)]
pub(super) struct InputKey {
    pub(super) format: ColorFormat,
    pub(super) color: Okhsl,
}
