//! APCA (Accessible Perceptual Contrast Algorithm), APCA-W3 0.0.98G-4g.
//!
//! Operates on gamma-encoded sRGB in `0.0..=1.0`; contrast is reported as a
//! signed `Lc` value, positive for dark-on-light and negative for
//! light-on-dark, with a magnitude of roughly `0..=106`.

use palette::Srgb;

const MAIN_TRC: f32 = 2.4;
const SOFT_CLAMP_THRESHOLD: f32 = 0.022;
const SOFT_CLAMP_EXPONENT: f32 = 1.414;
const RED_COEFFICIENT: f32 = 0.2126729;
const GREEN_COEFFICIENT: f32 = 0.7151522;
const BLUE_COEFFICIENT: f32 = 0.0721750;
const NORMAL_BACKGROUND_EXPONENT: f32 = 0.56;
const NORMAL_TEXT_EXPONENT: f32 = 0.57;
const REVERSE_BACKGROUND_EXPONENT: f32 = 0.65;
const REVERSE_TEXT_EXPONENT: f32 = 0.62;
const SCALE: f32 = 1.14;
const OFFSET: f32 = 0.027;
const LOW_CLIP: f32 = 0.1;
const PERCENT: f32 = 100.0;

/// Signed APCA lightness contrast between `text` and `background`.
pub fn contrast(text: Srgb, background: Srgb) -> f32 {
    let text_luminance = luminance(text);
    let background_luminance = luminance(background);

    if background_luminance > text_luminance {
        let sapc = (background_luminance.powf(NORMAL_BACKGROUND_EXPONENT)
            - text_luminance.powf(NORMAL_TEXT_EXPONENT))
            * SCALE;
        if sapc < LOW_CLIP {
            0.0
        } else {
            (sapc - OFFSET) * PERCENT
        }
    } else {
        let sapc = (background_luminance.powf(REVERSE_BACKGROUND_EXPONENT)
            - text_luminance.powf(REVERSE_TEXT_EXPONENT))
            * SCALE;
        if sapc > -LOW_CLIP {
            0.0
        } else {
            (sapc + OFFSET) * PERCENT
        }
    }
}

fn luminance(color: Srgb) -> f32 {
    let y = RED_COEFFICIENT * color.red.clamp(0.0, 1.0).powf(MAIN_TRC)
        + GREEN_COEFFICIENT * color.green.clamp(0.0, 1.0).powf(MAIN_TRC)
        + BLUE_COEFFICIENT * color.blue.clamp(0.0, 1.0).powf(MAIN_TRC);
    if y < SOFT_CLAMP_THRESHOLD {
        y + (SOFT_CLAMP_THRESHOLD - y).powf(SOFT_CLAMP_EXPONENT)
    } else {
        y
    }
}

#[cfg(test)]
mod tests {
    use super::contrast;
    use palette::Srgb;

    const EPSILON: f32 = 1e-3;

    #[test]
    fn black_on_white_is_maximum_contrast() {
        let lc = contrast(Srgb::new(0.0, 0.0, 0.0), Srgb::new(1.0, 1.0, 1.0));
        assert!(lc > 100.0, "expected Lc > 100, got {lc}");
        assert!(lc < 110.0, "expected Lc < 110, got {lc}");
    }

    #[test]
    fn white_on_black_is_reverse_polarity() {
        let lc = contrast(Srgb::new(1.0, 1.0, 1.0), Srgb::new(0.0, 0.0, 0.0));
        assert!(lc < -100.0, "expected Lc < -100, got {lc}");
    }

    #[test]
    fn identical_colors_have_no_contrast() {
        let color = Srgb::new(0.5, 0.5, 0.5);
        assert!(contrast(color, color).abs() < EPSILON);
    }
}
