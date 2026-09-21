use eframe::egui;

use crate::color::apca;
use crate::color::okhsl::Okhsl;
use crate::config::Theme;

const CONTRAST_LIGHTNESS_THRESHOLD: f32 = 0.5;
const TARGET_CONTRAST_LC: f32 = 90.0;
const CONTRAST_SEARCH_STEPS: u32 = 12;
const SURFACE_LIGHTNESS_SHIFT: f32 = 0.06;
const INACTIVE_FILL_ALPHA: u8 = 32;
const HOVERED_FILL_ALPHA: u8 = 60;
const ACTIVE_FILL_ALPHA: u8 = 96;
const SELECTION_FILL_ALPHA: u8 = 80;
const FIELD_FILL_ALPHA: u8 = 28;
const BORDER_ALPHA: u8 = 110;
const BORDER_WIDTH: f32 = 1.0;
const CORNER_RADIUS: u8 = 6;

pub fn color32(color: Okhsl) -> egui::Color32 {
    let [red, green, blue] = color.to_srgb8();
    egui::Color32::from_rgb(red, green, blue)
}

pub fn contrast_color32(color: Okhsl) -> egui::Color32 {
    color32(foreground(color))
}

fn foreground(background: Okhsl) -> Okhsl {
    let dark = Okhsl::new(background.hue(), background.saturation(), 0.0);
    let light = Okhsl::new(background.hue(), background.saturation(), 1.0);
    let dark_contrast = contrast_against(dark, background);
    let light_contrast = contrast_against(light, background);

    let (endpoint, best) = if dark_contrast >= light_contrast {
        (dark, dark_contrast)
    } else {
        (light, light_contrast)
    };
    if best <= TARGET_CONTRAST_LC {
        return endpoint;
    }

    // Contrast is highest at the endpoint and falls to zero at the
    // background's own lightness, so binary-search the smallest lightness
    // shift that still meets the target.
    let background_lightness = background.lightness();
    let endpoint_lightness = endpoint.lightness();
    let mut insufficient = 0.0;
    let mut sufficient = 1.0;
    for _ in 0..CONTRAST_SEARCH_STEPS {
        let step = (insufficient + sufficient) / 2.0;
        let lightness = background_lightness + (endpoint_lightness - background_lightness) * step;
        let candidate = Okhsl::new(background.hue(), background.saturation(), lightness);
        if contrast_against(candidate, background) >= TARGET_CONTRAST_LC {
            sufficient = step;
        } else {
            insufficient = step;
        }
    }

    let lightness = background_lightness + (endpoint_lightness - background_lightness) * sufficient;
    let searched = Okhsl::new(background.hue(), background.saturation(), lightness);
    if contrast_against(searched, background) >= TARGET_CONTRAST_LC {
        searched
    } else {
        endpoint
    }
}

fn contrast_against(color: Okhsl, background: Okhsl) -> f32 {
    apca::contrast(color.to_srgb(), background.to_srgb()).abs()
}

pub fn apply(ctx: &egui::Context, theme: Theme, color: Okhsl) {
    ctx.set_theme(theme_preference(theme));
    let visuals = visuals(ctx, theme, color);
    ctx.set_visuals_of(egui::Theme::Dark, visuals.clone());
    ctx.set_visuals_of(egui::Theme::Light, visuals);
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
    let surface = surface_color32(color);
    let border = egui::Stroke::new(BORDER_WIDTH, fill(foreground, BORDER_ALPHA));
    let corner = egui::CornerRadius::same(CORNER_RADIUS);

    visuals.window_corner_radius = corner;
    visuals.menu_corner_radius = corner;
    visuals.widgets.noninteractive.corner_radius = corner;
    visuals.widgets.inactive.corner_radius = corner;
    visuals.widgets.hovered.corner_radius = corner;
    visuals.widgets.active.corner_radius = corner;
    visuals.widgets.open.corner_radius = corner;

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
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(BORDER_WIDTH, foreground);
    visuals.widgets.inactive.bg_stroke = border;
    set_surfaces(&mut visuals, surface, border, foreground);
    visuals.selection.bg_fill = fill(foreground, SELECTION_FILL_ALPHA);
    visuals.selection.stroke.color = foreground;
    visuals
}

fn surface_color32(color: Okhsl) -> egui::Color32 {
    let lightness = color.lightness();
    let shifted = if lightness >= CONTRAST_LIGHTNESS_THRESHOLD {
        (lightness - SURFACE_LIGHTNESS_SHIFT).max(0.0)
    } else {
        (lightness + SURFACE_LIGHTNESS_SHIFT).min(1.0)
    };
    color32(Okhsl::new(color.hue(), color.saturation(), shifted))
}

fn set_surfaces(
    visuals: &mut egui::Visuals,
    surface: egui::Color32,
    border: egui::Stroke,
    foreground: egui::Color32,
) {
    visuals.window_fill = surface;
    visuals.panel_fill = surface;
    visuals.window_stroke = border;
    visuals.extreme_bg_color = fill(foreground, FIELD_FILL_ALPHA);
}

fn fill(foreground: egui::Color32, alpha: u8) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(foreground.r(), foreground.g(), foreground.b(), alpha)
}

#[cfg(test)]
mod tests {
    use super::contrast_color32;
    use crate::color::okhsl::Okhsl;
    use palette::Srgb;

    fn round_trip(color: Okhsl) -> Okhsl {
        let [red, green, blue] = color.to_srgb8();
        Okhsl::from_srgb(Srgb::new(
            f32::from(red) / 255.0,
            f32::from(green) / 255.0,
            f32::from(blue) / 255.0,
        ))
    }

    #[test]
    fn foreground_is_stable_across_an_srgb_round_trip() {
        let color = Okhsl::new(180.0, 0.5, 0.5);
        assert_eq!(contrast_color32(color), contrast_color32(round_trip(color)));
    }

    #[test]
    fn extreme_backgrounds_do_not_get_extreme_text() {
        let on_black = contrast_color32(Okhsl::new(0.0, 0.0, 0.0));
        let on_white = contrast_color32(Okhsl::new(0.0, 0.0, 1.0));
        assert!(on_black.r() > 16 && on_black.r() < 240);
        assert!(on_white.r() > 16 && on_white.r() < 240);
    }

    #[test]
    fn foreground_reaches_the_target_where_possible() {
        for (lightness, expect_light_text) in [(0.0, true), (1.0, false)] {
            let background = Okhsl::new(120.0, 0.5, lightness);
            let foreground = super::foreground(background);
            assert!(super::contrast_against(foreground, background) >= super::TARGET_CONTRAST_LC);
            assert_eq!(
                foreground.lightness() > 0.5,
                expect_light_text,
                "lightness {lightness}"
            );
        }
    }
}
