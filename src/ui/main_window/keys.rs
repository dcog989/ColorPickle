use eframe::egui;

use crate::color::okhsl::Okhsl;
use crate::color::{ColorFormat, Precision};
use crate::config::Theme;

#[derive(Clone, Copy, PartialEq)]
pub(super) struct ThemeKey {
    pub(super) theme: Theme,
    pub(super) system: Option<egui::Theme>,
}

#[derive(Clone, Copy, PartialEq)]
pub(super) struct InputKey {
    pub(super) format: ColorFormat,
    pub(super) precision: Precision,
    pub(super) color: Okhsl,
}
