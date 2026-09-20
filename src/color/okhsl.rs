//! Internal Okhsl color model.
//!
//! Backed by `palette`'s `Okhsl`, a port of Björn Ottosson's reference
//! implementation (MIT). Hue is in degrees `0..360`; saturation and lightness
//! are in `0.0..=1.0`.

use palette::{FromColor, IntoColor, Oklab, OklabHue, Srgb};

#[derive(Debug, Clone, Copy)]
pub struct Okhsl {
    inner: palette::Okhsl,
}

impl Okhsl {
    pub fn new(hue_degrees: f32, saturation: f32, lightness: f32) -> Self {
        Self {
            inner: palette::Okhsl::new(OklabHue::from_degrees(hue_degrees), saturation, lightness),
        }
    }

    pub fn from_srgb(color: Srgb) -> Self {
        Self {
            inner: Oklab::from_color(color).into_color(),
        }
    }

    pub fn to_srgb(self) -> Srgb {
        Srgb::from_color(Oklab::from_color(self.inner))
    }

    pub fn to_srgb8(self) -> [u8; 3] {
        let color: palette::Srgb<u8> = self.to_srgb().into_format();
        [color.red, color.green, color.blue]
    }

    pub fn hue(self) -> f32 {
        self.inner.hue.into_positive_degrees()
    }

    pub fn saturation(self) -> f32 {
        self.inner.saturation
    }

    pub fn lightness(self) -> f32 {
        self.inner.lightness
    }
}

#[cfg(test)]
mod tests {
    use super::Okhsl;
    use palette::Srgb;

    const EPSILON: f32 = 1e-3;

    const PRIMARIES: [(f32, f32, f32); 6] = [
        (1.0, 0.0, 0.0),
        (0.0, 1.0, 0.0),
        (0.0, 0.0, 1.0),
        (1.0, 1.0, 0.0),
        (0.0, 1.0, 1.0),
        (1.0, 0.0, 1.0),
    ];

    #[test]
    fn srgb_round_trips_through_okhsl() {
        for (r, g, b) in PRIMARIES {
            let round_tripped = Okhsl::from_srgb(Srgb::new(r, g, b)).to_srgb();
            assert!(
                (round_tripped.red - r).abs() < EPSILON,
                "red {r} vs {}",
                round_tripped.red
            );
            assert!(
                (round_tripped.green - g).abs() < EPSILON,
                "green {g} vs {}",
                round_tripped.green
            );
            assert!(
                (round_tripped.blue - b).abs() < EPSILON,
                "blue {b} vs {}",
                round_tripped.blue
            );
        }
    }

    #[test]
    fn achromatic_colors_have_no_saturation() {
        for level in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let okhsl = Okhsl::from_srgb(Srgb::new(level, level, level));
            assert!(okhsl.saturation() < EPSILON, "saturation for gray {level}");
        }
    }

    #[test]
    fn hue_is_reported_in_positive_degrees() {
        assert!((Okhsl::new(270.0, 1.0, 0.5).hue() - 270.0).abs() < EPSILON);
        assert!((Okhsl::new(30.0, 1.0, 0.5).hue() - 30.0).abs() < EPSILON);
    }

    #[test]
    fn neutral_lightness_is_a_gray() {
        let [r, g, b] = Okhsl::new(180.0, 0.0, 0.5).to_srgb8();
        assert_eq!(r, g);
        assert_eq!(g, b);
        assert!((100..=140).contains(&r), "unexpected mid-gray {r}");
    }
}
