use crate::color::okhsl::Okhsl;

const FULL_TURN_DEGREES: f32 = 360.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Harmony {
    #[default]
    Complementary,
    SplitComplementary,
    Analogous,
    Triadic,
    Tetradic,
    Rectangle,
}

impl Harmony {
    pub const ALL: [Self; 6] = [
        Self::Complementary,
        Self::SplitComplementary,
        Self::Analogous,
        Self::Triadic,
        Self::Tetradic,
        Self::Rectangle,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Complementary => "Complementary",
            Self::SplitComplementary => "Split complementary",
            Self::Analogous => "Analogous",
            Self::Triadic => "Triadic",
            Self::Tetradic => "Tetradic",
            Self::Rectangle => "Rectangle",
        }
    }

    pub const fn offsets(self) -> &'static [f32] {
        match self {
            Self::Complementary => &[0.0, 180.0],
            Self::SplitComplementary => &[0.0, 150.0, 210.0],
            Self::Analogous => &[330.0, 0.0, 30.0],
            Self::Triadic => &[0.0, 120.0, 240.0],
            Self::Tetradic => &[0.0, 90.0, 180.0, 270.0],
            Self::Rectangle => &[0.0, 60.0, 180.0, 240.0],
        }
    }

    pub fn swatches(self, color: Okhsl) -> Vec<Okhsl> {
        self.offsets()
            .iter()
            .map(|&offset| {
                let hue = (color.hue() + offset).rem_euclid(FULL_TURN_DEGREES);
                Okhsl::new(hue, color.saturation(), color.lightness())
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::Harmony;
    use crate::color::okhsl::Okhsl;

    #[test]
    fn swatches_rotate_and_wrap_hue() {
        let color = Okhsl::new(350.0, 0.5, 0.5);
        let analogous = Harmony::Analogous.swatches(color);
        assert!((analogous[0].hue() - 320.0).abs() < 1e-3);
        assert!((analogous[1].hue() - 350.0).abs() < 1e-3);
        assert!((analogous[2].hue() - 20.0).abs() < 1e-3);
    }

    #[test]
    fn complementary_keeps_saturation_and_lightness() {
        let color = Okhsl::new(30.0, 0.4, 0.6);
        let swatches = Harmony::Complementary.swatches(color);
        assert_eq!(swatches.len(), 2);
        assert!((swatches[1].hue() - 210.0).abs() < 1e-3);
        assert!((swatches[1].saturation() - 0.4).abs() < 1e-3);
        assert!((swatches[1].lightness() - 0.6).abs() < 1e-3);
    }
}
