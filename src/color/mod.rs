pub mod apca;
pub mod harmony;
pub mod okhsl;
pub mod parse;

use palette::{FromColor, Hsl, Lab, Oklab, Oklch, Srgb};
use serde::{Deserialize, Serialize};

use self::okhsl::Okhsl;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorFormat {
    #[default]
    Hex,
    Rgb,
    Hsl,
    Okhsl,
    Oklch,
    Oklab,
    Cmyk,
    Cielab,
}

/// How many decimals each format prints. Mirrors Color-Picker-Palette: precise
/// keeps decimals (two for the percentage and 8-bit channels, one for the hue
/// in degrees, four for the 0..1 perceptual channels), practical rounds to
/// whole numbers (two for the perceptual channels, whose 0..1 scale would
/// otherwise collapse).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Precision {
    #[default]
    Practical,
    Precise,
}

impl Precision {
    pub const ALL: [Self; 2] = [Self::Practical, Self::Precise];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Practical => "Practical",
            Self::Precise => "Precise",
        }
    }

    const fn hue(self) -> usize {
        match self {
            Self::Practical => 0,
            Self::Precise => 1,
        }
    }

    const fn fraction(self) -> usize {
        match self {
            Self::Practical => 2,
            Self::Precise => 4,
        }
    }

    const fn channel(self) -> usize {
        match self {
            Self::Practical => 0,
            Self::Precise => 2,
        }
    }
}

impl ColorFormat {
    pub const ALL: [Self; 8] = [
        Self::Hex,
        Self::Rgb,
        Self::Hsl,
        Self::Okhsl,
        Self::Oklch,
        Self::Oklab,
        Self::Cmyk,
        Self::Cielab,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Hex => "HEX",
            Self::Rgb => "RGB",
            Self::Hsl => "HSL",
            Self::Okhsl => "Okhsl",
            Self::Oklch => "Oklch",
            Self::Oklab => "Oklab",
            Self::Cmyk => "CMYK",
            Self::Cielab => "CIELAB",
        }
    }

    pub fn format(self, color: Okhsl, precision: Precision) -> String {
        let srgb = color.to_srgb();
        match self {
            Self::Hex => format_hex(srgb),
            Self::Rgb => format!(
                "rgb({} {} {})",
                srgb_channel(srgb.red, precision.channel()),
                srgb_channel(srgb.green, precision.channel()),
                srgb_channel(srgb.blue, precision.channel())
            ),
            Self::Hsl => {
                let hsl = Hsl::from_color(srgb);
                format!(
                    "hsl({} {}% {}%)",
                    decimals(hsl.hue.into_positive_degrees(), precision.hue()),
                    decimals(hsl.saturation * 100.0, precision.channel()),
                    decimals(hsl.lightness * 100.0, precision.channel())
                )
            }
            Self::Okhsl => format!(
                "okhsl({} {} {})",
                decimals(color.hue(), precision.hue()),
                decimals(color.saturation(), precision.fraction()),
                decimals(color.lightness(), precision.fraction())
            ),
            Self::Oklch => {
                let oklch = Oklch::from_color(Oklab::from_color(srgb));
                format!(
                    "oklch({} {} {})",
                    decimals(oklch.l, precision.fraction()),
                    decimals(oklch.chroma, precision.fraction()),
                    decimals(oklch.hue.into_positive_degrees(), precision.hue())
                )
            }
            Self::Oklab => {
                let oklab = Oklab::from_color(srgb);
                format!(
                    "oklab({} {} {})",
                    decimals(oklab.l, precision.fraction()),
                    decimals(oklab.a, precision.fraction()),
                    decimals(oklab.b, precision.fraction())
                )
            }
            Self::Cmyk => format_cmyk(srgb, precision.channel()),
            Self::Cielab => {
                let lab: Lab = Lab::from_color(srgb);
                format!(
                    "lab({} {} {})",
                    decimals(lab.l, precision.fraction()),
                    decimals(lab.a, precision.fraction()),
                    decimals(lab.b, precision.fraction())
                )
            }
        }
    }
}

fn decimals(value: f32, places: usize) -> String {
    let formatted = format!("{value:.places$}");
    let text = match formatted.split_once('.') {
        Some((integer, fraction)) => {
            let fraction = fraction.trim_end_matches('0');
            if fraction.is_empty() {
                integer.to_string()
            } else {
                format!("{integer}.{fraction}")
            }
        }
        None => formatted,
    };
    if text == "-0" { "0".to_string() } else { text }
}

fn to_srgb8(srgb: Srgb) -> [u8; 3] {
    let color: palette::Srgb<u8> = srgb.into_format();
    [color.red, color.green, color.blue]
}

const CHANNEL_MAX: f32 = 255.0;

fn srgb_channel(value: f32, places: usize) -> String {
    decimals(value.clamp(0.0, 1.0) * CHANNEL_MAX, places)
}

fn format_hex(srgb: Srgb) -> String {
    let [red, green, blue] = to_srgb8(srgb);
    format!("#{red:02X}{green:02X}{blue:02X}")
}

fn format_cmyk(srgb: Srgb, places: usize) -> String {
    let red = srgb.red.clamp(0.0, 1.0);
    let green = srgb.green.clamp(0.0, 1.0);
    let blue = srgb.blue.clamp(0.0, 1.0);

    let black = 1.0 - red.max(green).max(blue);
    let denominator = 1.0 - black;
    let (cyan, magenta, yellow) = if denominator <= 0.0 {
        (0.0, 0.0, 0.0)
    } else {
        (
            (1.0 - red - black) / denominator,
            (1.0 - green - black) / denominator,
            (1.0 - blue - black) / denominator,
        )
    };

    format!(
        "cmyk({}% {}% {}% {}%)",
        decimals(cyan * 100.0, places),
        decimals(magenta * 100.0, places),
        decimals(yellow * 100.0, places),
        decimals(black * 100.0, places)
    )
}

#[cfg(test)]
mod tests {
    use super::{ColorFormat, Precision, decimals, format_cmyk, format_hex, srgb_channel};
    use crate::color::okhsl::Okhsl;
    use palette::Srgb;

    #[test]
    fn decimals_trims_and_avoids_negative_zero() {
        assert_eq!(decimals(0.5, 3), "0.5");
        assert_eq!(decimals(180.0, 1), "180");
        assert_eq!(decimals(-0.0001, 3), "0");
        assert_eq!(decimals(-0.0, 3), "0");
        assert_eq!(decimals(-2.25, 3), "-2.25");
    }

    #[test]
    fn hex_uses_uppercase_pairs() {
        assert_eq!(format_hex(Srgb::new(1.0, 0.0, 0.0)), "#FF0000");
        assert_eq!(format_hex(Srgb::new(1.0, 0.5, 0.0)), "#FF8000");
    }

    #[test]
    fn cmyk_maps_red_and_black() {
        assert_eq!(
            format_cmyk(Srgb::new(1.0, 0.0, 0.0), 0),
            "cmyk(0% 100% 100% 0%)"
        );
        assert_eq!(
            format_cmyk(Srgb::new(0.0, 0.0, 0.0), 0),
            "cmyk(0% 0% 0% 100%)"
        );
    }

    #[test]
    fn srgb_channel_honours_places_and_clamps() {
        assert_eq!(srgb_channel(0.5, 2), "127.5");
        assert_eq!(srgb_channel(0.5, 0), "128");
        assert_eq!(srgb_channel(1.2, 2), "255");
        assert_eq!(srgb_channel(-0.1, 2), "0");
    }

    #[test]
    fn channel_formats_honour_precision() {
        let color = Okhsl::from_srgb(Srgb::new(0.123, 0.456, 0.789));
        for format in [ColorFormat::Rgb, ColorFormat::Hsl, ColorFormat::Cmyk] {
            assert!(
                !format.format(color, Precision::Practical).contains('.'),
                "{format:?} practical"
            );
            assert!(
                format.format(color, Precision::Precise).contains('.'),
                "{format:?} precise"
            );
        }
    }

    #[test]
    fn okhsl_format_honours_precision() {
        let color = Okhsl::new(336.34, 0.287, 0.4375);
        assert_eq!(
            ColorFormat::Okhsl.format(color, Precision::Practical),
            "okhsl(336 0.29 0.44)"
        );
        assert_eq!(
            ColorFormat::Okhsl.format(color, Precision::Precise),
            "okhsl(336.3 0.287 0.4375)"
        );
    }

    fn component(text: &str, index: usize) -> f32 {
        text.split_once('(')
            .unwrap()
            .1
            .split_whitespace()
            .nth(index - 1)
            .unwrap()
            .trim_end_matches([')', '%'])
            .parse()
            .unwrap()
    }

    #[test]
    fn formatted_hues_are_positive_degrees() {
        for srgb in [
            Srgb::new(0.0, 1.0, 1.0),
            Srgb::new(0.5, 0.2, 0.9),
            Srgb::new(0.1, 0.8, 0.4),
        ] {
            let color = Okhsl::from_srgb(srgb);
            let hsl = ColorFormat::Hsl.format(color, Precision::Practical);
            assert!((0.0..360.0).contains(&component(&hsl, 1)), "{hsl}");
            let oklch = ColorFormat::Oklch.format(color, Precision::Practical);
            assert!((0.0..360.0).contains(&component(&oklch, 3)), "{oklch}");
        }
    }
}
