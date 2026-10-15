use eframe::egui;

use crate::color::apca;
use crate::color::okhsl::Okhsl;
use crate::config::Theme;

const CONTRAST_LIGHTNESS_THRESHOLD: f32 = 0.5;
const TARGET_CONTRAST_LC: f32 = 90.0;
const MIN_CONTRAST_LC: f32 = 75.0;
const MIN_FILL_LIGHTNESS: f32 = 0.2;
const CONTRAST_SEARCH_STEPS: u32 = 12;
const SURFACE_LIGHTNESS_SHIFT: f32 = 0.06;
const INACTIVE_SHADE: f32 = 32.0 / 255.0;
const HOVERED_SHADE: f32 = 60.0 / 255.0;
const ACTIVE_SHADE: f32 = 96.0 / 255.0;
const OPEN_SHADE: f32 = 48.0 / 255.0;
const SELECTION_SHADE: f32 = 80.0 / 255.0;
const FIELD_SHADE: f32 = 28.0 / 255.0;
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

/// Clamp a UI fill so text on it keeps at least [`MIN_CONTRAST_LC`], and keep
/// it off pure black. UI fills only — the colour swatch always shows the picked
/// colour, so it is never clamped.
fn clamp_fill(color: Okhsl, foreground: Okhsl) -> Okhsl {
    let hue = color.hue();
    let saturation = color.saturation();
    let lightness = color.lightness().max(MIN_FILL_LIGHTNESS);
    let lifted = Okhsl::new(hue, saturation, lightness);
    if contrast_against(foreground, lifted) >= MIN_CONTRAST_LC {
        return lifted;
    }

    // Contrast rises monotonically as the fill moves away from the text, so
    // binary-search the smallest shift that clears the floor.
    let away = if foreground.lightness() > CONTRAST_LIGHTNESS_THRESHOLD {
        0.0
    } else {
        1.0
    };
    let mut below = 0.0;
    let mut above = 1.0;
    for _ in 0..CONTRAST_SEARCH_STEPS {
        let step = (below + above) / 2.0;
        let sample = (lightness + (away - lightness) * step).clamp(0.0, 1.0);
        if contrast_against(foreground, Okhsl::new(hue, saturation, sample)) >= MIN_CONTRAST_LC {
            above = step;
        } else {
            below = step;
        }
    }
    let lightness = (lightness + (away - lightness) * above).clamp(0.0, 1.0);
    Okhsl::new(hue, saturation, lightness)
}

/// A widget fill: the swatch shifted `fraction` of the way toward the text,
/// then clamped so text on the fill stays readable.
fn fill_color(background: Okhsl, foreground: Okhsl, fraction: f32) -> Okhsl {
    let lightness =
        background.lightness() + (foreground.lightness() - background.lightness()) * fraction;
    clamp_fill(
        Okhsl::new(background.hue(), background.saturation(), lightness),
        foreground,
    )
}

/// A popup/tooltip surface: the swatch nudged toward mid grey, then clamped.
fn surface_color(color: Okhsl, foreground: Okhsl) -> Okhsl {
    let lightness = color.lightness();
    let shifted = if lightness >= CONTRAST_LIGHTNESS_THRESHOLD {
        (lightness - SURFACE_LIGHTNESS_SHIFT).max(0.0)
    } else {
        (lightness + SURFACE_LIGHTNESS_SHIFT).min(1.0)
    };
    clamp_fill(
        Okhsl::new(color.hue(), color.saturation(), shifted),
        foreground,
    )
}

pub fn apply(ctx: &egui::Context, theme: Theme, color: Okhsl) {
    ctx.set_theme(theme_preference(theme));
    let mut visuals = base_visuals(ctx, theme);
    apply_color(&mut visuals, color);
    ctx.set_visuals_of(egui::Theme::Dark, visuals.clone());
    ctx.set_visuals_of(egui::Theme::Light, visuals);
}

pub fn apply_color_to_current(ctx: &egui::Context, color: Okhsl) {
    for theme in [egui::Theme::Dark, egui::Theme::Light] {
        let mut visuals = ctx.style_of(theme).visuals.clone();
        apply_color(&mut visuals, color);
        ctx.set_visuals_of(theme, visuals);
    }
}

fn theme_preference(theme: Theme) -> egui::ThemePreference {
    match theme {
        Theme::System => egui::ThemePreference::System,
        Theme::Dark => egui::ThemePreference::Dark,
        Theme::Light => egui::ThemePreference::Light,
    }
}

fn base_visuals(ctx: &egui::Context, theme: Theme) -> egui::Visuals {
    let mut visuals = match theme {
        Theme::Dark => egui::Visuals::dark(),
        Theme::Light => egui::Visuals::light(),
        Theme::System => match ctx.system_theme() {
            Some(egui::Theme::Light) => egui::Visuals::light(),
            _ => egui::Visuals::dark(),
        },
    };

    let corner = egui::CornerRadius::same(CORNER_RADIUS);
    visuals.window_corner_radius = corner;
    visuals.menu_corner_radius = corner;
    visuals.widgets.noninteractive.corner_radius = corner;
    visuals.widgets.inactive.corner_radius = corner;
    visuals.widgets.hovered.corner_radius = corner;
    visuals.widgets.active.corner_radius = corner;
    visuals.widgets.open.corner_radius = corner;
    visuals
}

fn apply_color(visuals: &mut egui::Visuals, color: Okhsl) {
    let foreground_okhsl = foreground(color);
    let foreground = color32(foreground_okhsl);
    let surface = color32(surface_color(color, foreground_okhsl));
    let border = egui::Stroke::new(BORDER_WIDTH, fill(foreground, BORDER_ALPHA));
    let shade = |fraction: f32| color32(fill_color(color, foreground_okhsl, fraction));

    visuals.override_text_color = Some(foreground);
    visuals.widgets.noninteractive.fg_stroke.color = foreground;
    visuals.widgets.inactive.fg_stroke.color = foreground;
    visuals.widgets.hovered.fg_stroke.color = foreground;
    visuals.widgets.active.fg_stroke.color = foreground;
    visuals.widgets.open.fg_stroke.color = foreground;
    visuals.widgets.inactive.weak_bg_fill = shade(INACTIVE_SHADE);
    visuals.widgets.hovered.weak_bg_fill = shade(HOVERED_SHADE);
    visuals.widgets.active.weak_bg_fill = shade(ACTIVE_SHADE);
    visuals.widgets.open.weak_bg_fill = shade(OPEN_SHADE);
    visuals.widgets.inactive.bg_fill = shade(INACTIVE_SHADE);
    visuals.widgets.hovered.bg_fill = shade(HOVERED_SHADE);
    visuals.widgets.active.bg_fill = shade(ACTIVE_SHADE);
    visuals.widgets.open.bg_fill = shade(OPEN_SHADE);
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(BORDER_WIDTH, foreground);
    visuals.widgets.inactive.bg_stroke = border;
    set_surfaces(visuals, surface, border, shade(FIELD_SHADE));
    visuals.selection.bg_fill = shade(SELECTION_SHADE);
    visuals.selection.stroke.color = foreground;
}

fn set_surfaces(
    visuals: &mut egui::Visuals,
    surface: egui::Color32,
    border: egui::Stroke,
    field: egui::Color32,
) {
    visuals.window_fill = surface;
    visuals.panel_fill = surface;
    visuals.window_stroke = border;
    visuals.extreme_bg_color = field;
}

fn fill(color: egui::Color32, alpha: u8) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

#[cfg(test)]
mod tests {
    use palette::Srgb;

    use super::contrast_color32;
    use crate::color::okhsl::Okhsl;

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

    #[test]
    fn fills_clear_the_floor_for_dead_band_colours() {
        for lightness in [0.55, 0.65, 0.70, 0.75] {
            let color = Okhsl::new(116.0, 0.78, lightness);
            let foreground = super::foreground(color);
            for fraction in [
                super::FIELD_SHADE,
                super::INACTIVE_SHADE,
                super::HOVERED_SHADE,
                super::ACTIVE_SHADE,
                super::OPEN_SHADE,
                super::SELECTION_SHADE,
            ] {
                let fill = super::fill_color(color, foreground, fraction);
                assert!(
                    super::contrast_against(foreground, fill) >= super::MIN_CONTRAST_LC,
                    "lightness {lightness} fraction {fraction}"
                );
            }
        }
    }

    #[test]
    fn surfaces_keep_contrast_against_the_text() {
        for lightness in [0.0, 0.15, 0.43, 0.65, 0.80, 0.95] {
            let color = Okhsl::new(116.0, 0.78, lightness);
            let foreground = super::foreground(color);
            let surface = super::surface_color(color, foreground);
            assert!(
                super::contrast_against(foreground, surface) >= super::MIN_CONTRAST_LC,
                "lightness {lightness}"
            );
        }
    }

    #[test]
    fn fills_stay_off_black() {
        let color = Okhsl::new(116.0, 0.78, 0.0);
        let foreground = super::foreground(color);
        for fraction in [super::FIELD_SHADE, super::ACTIVE_SHADE] {
            let fill = super::fill_color(color, foreground, fraction);
            assert!(fill.lightness() >= super::MIN_FILL_LIGHTNESS);
        }
    }
}
