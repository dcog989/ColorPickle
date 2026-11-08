use eframe::egui;

use crate::color::apca;
use crate::color::okhsl::Okhsl;

const CONTRAST_LIGHTNESS_THRESHOLD: f32 = 0.5;
const TARGET_CONTRAST_LC: f32 = 90.0;
/// Text on a resting surface.
const MIN_TEXT_CONTRAST_LC: f32 = 75.0;
const MIN_FILL_LIGHTNESS: f32 = 0.2;
const CONTRAST_SEARCH_STEPS: u32 = 12;
const SURFACE_LIGHTNESS_SHIFT: f32 = 0.06;
const INACTIVE_SHADE: f32 = 30.0 / 255.0;
const HOVERED_SHADE: f32 = 77.0 / 255.0;
const ACTIVE_SHADE: f32 = 115.0 / 255.0;
const OPEN_SHADE: f32 = 64.0 / 255.0;
const FIELD_SHADE: f32 = 28.0 / 255.0;
const BORDER_ALPHA: u8 = 110;
const HOVERED_BORDER_ALPHA: u8 = 200;
const ACTIVE_BORDER_ALPHA: u8 = 255;
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

/// Clamp a UI fill so text on it keeps at least `floor` contrast, and keep it
/// off pure black. UI fills only — the colour swatch always shows the picked
/// colour, so it is never clamped.
fn clamp_fill(color: Okhsl, foreground: Okhsl, floor: f32) -> Okhsl {
    let hue = color.hue();
    let saturation = color.saturation();
    let lightness = color.lightness().max(MIN_FILL_LIGHTNESS);
    let lifted = Okhsl::new(hue, saturation, lightness);
    if contrast_against(foreground, lifted) >= floor {
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
        if contrast_against(foreground, Okhsl::new(hue, saturation, sample)) >= floor {
            above = step;
        } else {
            below = step;
        }
    }
    let lightness = (lightness + (away - lightness) * above).clamp(0.0, 1.0);
    Okhsl::new(hue, saturation, lightness)
}

/// An interactive fill: the surface pulled `fraction` of the way *away* from the
/// text, so every state adds separation instead of spending it. The opposite
/// direction has no room: a surface that meets the text floor loses contrast as
/// soon as it moves toward the text, which flattened hover and active into idle.
fn shade_color(surface: Okhsl, foreground: Okhsl, fraction: f32) -> Okhsl {
    let away = if foreground.lightness() > CONTRAST_LIGHTNESS_THRESHOLD {
        0.0
    } else {
        1.0
    };
    let lightness = surface.lightness() + (away - surface.lightness()) * fraction;
    Okhsl::new(surface.hue(), surface.saturation(), lightness)
}

/// A fill that sits behind text for longer than a hover: the surface shifted
/// `fraction` of the way toward the text, clamped so the text stays readable.
fn fill_color(surface: Okhsl, foreground: Okhsl, fraction: f32, floor: f32) -> Okhsl {
    let lightness = surface.lightness() + (foreground.lightness() - surface.lightness()) * fraction;
    clamp_fill(
        Okhsl::new(surface.hue(), surface.saturation(), lightness),
        foreground,
        floor,
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
        MIN_TEXT_CONTRAST_LC,
    )
}

/// Installs the visuals for both egui themes. `apply_color` overrides every
/// colour this UI draws, so the light and dark slots end up identical and the
/// system preference only decides which slot is read.
pub fn install(ctx: &egui::Context, color: Okhsl) {
    let mut visuals = base_visuals();
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

fn base_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();

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
    let surface_okhsl = surface_color(color, foreground_okhsl);
    let surface = color32(surface_okhsl);
    let border = egui::Stroke::new(BORDER_WIDTH, fill(foreground, BORDER_ALPHA));
    let hovered_border = egui::Stroke::new(BORDER_WIDTH, fill(foreground, HOVERED_BORDER_ALPHA));
    let active_border = egui::Stroke::new(BORDER_WIDTH, fill(foreground, ACTIVE_BORDER_ALPHA));
    let shade = |fraction: f32| color32(shade_color(surface_okhsl, foreground_okhsl, fraction));
    let readable = |fraction: f32| {
        color32(fill_color(
            surface_okhsl,
            foreground_okhsl,
            fraction,
            MIN_TEXT_CONTRAST_LC,
        ))
    };

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
    visuals.widgets.hovered.bg_stroke = hovered_border;
    visuals.widgets.active.bg_stroke = active_border;
    visuals.widgets.open.bg_stroke = hovered_border;
    set_surfaces(visuals, surface, border, readable(FIELD_SHADE));
    // Selected text inverts rather than tints: the field already sits at the
    // text contrast floor, so any tinted selection clamps to the same fill and
    // disappears.
    visuals.selection.bg_fill = foreground;
    visuals.selection.stroke.color = surface;
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
    use eframe::egui;
    use palette::Srgb;

    use super::contrast_color32;
    use crate::color::okhsl::Okhsl;

    /// Smallest sRGB channel step that still reads as a distinct fill.
    const MIN_HIGHLIGHT_DELTA: u8 = 10;

    /// Straight alpha composite; precise enough for a "is it visible" check.
    fn over(top: egui::Color32, bottom: egui::Color32) -> egui::Color32 {
        let alpha = f32::from(top.a()) / 255.0;
        let mix = |top: u8, bottom: u8| -> u8 {
            (f32::from(top) * alpha + f32::from(bottom) * (1.0 - alpha)).round() as u8
        };
        egui::Color32::from_rgb(
            mix(top.r(), bottom.r()),
            mix(top.g(), bottom.g()),
            mix(top.b(), bottom.b()),
        )
    }

    fn channel_delta(a: egui::Color32, b: egui::Color32) -> u8 {
        [a.r(), a.g(), a.b()]
            .into_iter()
            .zip([b.r(), b.g(), b.b()])
            .map(|(left, right)| left.abs_diff(right))
            .max()
            .unwrap()
    }

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
            let surface = super::surface_color(color, foreground);
            for fraction in [
                super::INACTIVE_SHADE,
                super::HOVERED_SHADE,
                super::ACTIVE_SHADE,
                super::OPEN_SHADE,
            ] {
                let fill = super::shade_color(surface, foreground, fraction);
                assert!(
                    super::contrast_against(foreground, fill)
                        >= super::contrast_against(foreground, surface),
                    "lightness {lightness} fraction {fraction}"
                );
            }
            let fraction = super::FIELD_SHADE;
            let fill =
                super::fill_color(surface, foreground, fraction, super::MIN_TEXT_CONTRAST_LC);
            let floor =
                super::MIN_TEXT_CONTRAST_LC.min(super::contrast_against(foreground, surface));
            assert!(
                super::contrast_against(foreground, fill) >= floor,
                "lightness {lightness} fraction {fraction}"
            );
        }
    }

    #[test]
    fn surfaces_keep_contrast_against_the_text() {
        for lightness in [0.0, 0.15, 0.43, 0.65, 0.80, 0.95] {
            let color = Okhsl::new(116.0, 0.78, lightness);
            let foreground = super::foreground(color);
            let surface = super::surface_color(color, foreground);
            assert!(
                super::contrast_against(foreground, surface) >= super::MIN_TEXT_CONTRAST_LC,
                "lightness {lightness}"
            );
        }
    }

    #[test]
    fn hover_and_active_stand_out_from_the_surface() {
        for lightness in [0.0, 0.05, 0.2, 0.35, 0.5, 0.65, 0.8, 0.95, 1.0] {
            for saturation in [0.0, 0.78] {
                let color = Okhsl::new(116.0, saturation, lightness);
                let mut visuals = egui::Visuals::dark();
                super::apply_color(&mut visuals, color);
                let surface = visuals.window_fill;
                let steps = [
                    visuals.widgets.inactive.weak_bg_fill,
                    visuals.widgets.hovered.weak_bg_fill,
                    visuals.widgets.active.weak_bg_fill,
                ]
                .map(|fill| channel_delta(surface, fill));
                assert!(
                    steps[0] < steps[1] && steps[1] < steps[2],
                    "lightness {lightness} saturation {saturation}: steps {steps:?}"
                );

                let borders = [
                    visuals.widgets.inactive.bg_stroke.color,
                    visuals.widgets.hovered.bg_stroke.color,
                    visuals.widgets.active.bg_stroke.color,
                ];
                let composited =
                    borders.map(|border| channel_delta(surface, over(border, surface)));
                assert!(
                    composited[1] > composited[0] && composited[2] > composited[1],
                    "lightness {lightness} saturation {saturation}: borders {borders:?}"
                );
                // Near white or black the fill has almost no room left, so the
                // border has to carry the state on its own.
                assert!(
                    steps[1].max(composited[1]) >= MIN_HIGHLIGHT_DELTA,
                    "visible lightness {lightness} saturation {saturation}: surface {surface:?} steps {steps:?} composite {composited:?}"
                );
            }
        }
    }

    #[test]
    fn text_selection_stands_out_from_the_field() {
        for lightness in [0.0, 0.2, 0.5, 0.65, 0.8, 1.0] {
            for saturation in [0.0, 0.78] {
                let color = Okhsl::new(116.0, saturation, lightness);
                let mut visuals = egui::Visuals::dark();
                super::apply_color(&mut visuals, color);
                assert!(
                    channel_delta(visuals.selection.bg_fill, visuals.extreme_bg_color)
                        >= MIN_HIGHLIGHT_DELTA,
                    "lightness {lightness} saturation {saturation}"
                );
            }
        }
    }

    #[test]
    fn fills_stay_off_black() {
        let color = Okhsl::new(116.0, 0.78, 0.0);
        let foreground = super::foreground(color);
        let surface = super::surface_color(color, foreground);
        let fill = super::fill_color(
            surface,
            foreground,
            super::FIELD_SHADE,
            super::MIN_TEXT_CONTRAST_LC,
        );
        assert!(fill.lightness() >= super::MIN_FILL_LIGHTNESS);

        let widget = super::shade_color(surface, foreground, super::ACTIVE_SHADE);
        assert!(widget.lightness() >= super::MIN_FILL_LIGHTNESS / 2.0);
    }
}
