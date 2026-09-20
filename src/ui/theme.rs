use eframe::egui;

use crate::color::okhsl::Okhsl;
use crate::config::Theme;

const CONTRAST_LIGHTNESS_THRESHOLD: f32 = 0.5;
const CONTRAST_LIGHTNESS_SHIFT: f32 = 0.4;
const INACTIVE_FILL_ALPHA: u8 = 32;
const HOVERED_FILL_ALPHA: u8 = 60;
const ACTIVE_FILL_ALPHA: u8 = 96;
const SELECTION_FILL_ALPHA: u8 = 80;

pub fn color32(color: Okhsl) -> egui::Color32 {
    let [red, green, blue] = color.to_srgb8();
    egui::Color32::from_rgb(red, green, blue)
}

pub fn contrast_color32(color: Okhsl) -> egui::Color32 {
    let lightness = color.lightness();
    let shifted = if lightness >= CONTRAST_LIGHTNESS_THRESHOLD {
        (lightness - CONTRAST_LIGHTNESS_SHIFT).max(0.0)
    } else {
        (lightness + CONTRAST_LIGHTNESS_SHIFT).min(1.0)
    };
    color32(Okhsl::new(color.hue(), color.saturation(), shifted))
}

pub fn apply(ctx: &egui::Context, theme: Theme, color: Okhsl) {
    ctx.set_theme(theme_preference(theme));
    ctx.set_visuals(visuals(ctx, theme, color));
}

fn theme_preference(theme: Theme) -> egui::ThemePreference {
    match theme {
        Theme::System => egui::ThemePreference::System,
        Theme::Dark => egui::ThemePreference::Dark,
        Theme::Light => egui::ThemePreference::Light,
    }
}

fn visuals(ctx: &egui::Context, theme: Theme, color: Okhsl) -> egui::Visuals {
    let mut visuals = match theme {
        Theme::Dark => egui::Visuals::dark(),
        Theme::Light => egui::Visuals::light(),
        Theme::System => match ctx.system_theme() {
            Some(egui::Theme::Light) => egui::Visuals::light(),
            _ => egui::Visuals::dark(),
        },
    };

    let foreground = contrast_color32(color);
    visuals.override_text_color = Some(foreground);
    visuals.widgets.noninteractive.fg_stroke.color = foreground;
    visuals.widgets.inactive.fg_stroke.color = foreground;
    visuals.widgets.hovered.fg_stroke.color = foreground;
    visuals.widgets.active.fg_stroke.color = foreground;
    visuals.widgets.open.fg_stroke.color = foreground;
    visuals.widgets.inactive.weak_bg_fill = fill(foreground, INACTIVE_FILL_ALPHA);
    visuals.widgets.hovered.weak_bg_fill = fill(foreground, HOVERED_FILL_ALPHA);
    visuals.widgets.active.weak_bg_fill = fill(foreground, ACTIVE_FILL_ALPHA);
    visuals.widgets.inactive.bg_fill = fill(foreground, INACTIVE_FILL_ALPHA);
    visuals.widgets.hovered.bg_fill = fill(foreground, HOVERED_FILL_ALPHA);
    visuals.widgets.active.bg_fill = fill(foreground, ACTIVE_FILL_ALPHA);
    visuals.selection.bg_fill = fill(foreground, SELECTION_FILL_ALPHA);
    visuals.selection.stroke.color = foreground;
    visuals
}

fn fill(foreground: egui::Color32, alpha: u8) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(foreground.r(), foreground.g(), foreground.b(), alpha)
}
