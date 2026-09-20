pub mod okhsl;
pub mod parse;

use clap::ValueEnum;
use palette::{FromColor, Hsl, Lab, Oklab, Oklch, Srgb};
use serde::{Deserialize, Serialize};

use self::okhsl::Okhsl;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum, Serialize, Deserialize)]
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

    pub fn format(self, color: Okhsl) -> String {
        let srgb = color.to_srgb();
        match self {
            Self::Hex => format_hex(srgb),
            Self::Rgb => {
                let [red, green, blue] = to_srgb8(srgb);
                format!("rgb({red}, {green}, {blue})")
            }
            Self::Hsl => {
                let hsl = Hsl::from_color(srgb);
                format!(
                    "hsl({:.0}, {:.0}%, {:.0}%)",
                    hsl.hue.into_positive_degrees(),
                    hsl.saturation * 100.0,
                    hsl.lightness * 100.0
                )
            }
            Self::Okhsl => format!(
                "okhsl({:.1}, {:.3}, {:.3})",
                color.hue(),
                color.saturation(),
                color.lightness()
            ),
            Self::Oklch => {
                let oklch = Oklch::from_color(Oklab::from_color(srgb));
                format!(
                    "oklch({:.3}, {:.3}, {:.1})",
                    oklch.l,
                    oklch.chroma,
                    oklch.hue.into_positive_degrees()
                )
            }
            Self::Oklab => {
                let oklab = Oklab::from_color(srgb);
                format!("oklab({:.3}, {:.3}, {:.3})", oklab.l, oklab.a, oklab.b)
            }
            Self::Cmyk => format_cmyk(srgb),
            Self::Cielab => {
                let lab: Lab = Lab::from_color(srgb);
                format!("lab({:.1}, {:.1}, {:.1})", lab.l, lab.a, lab.b)
            }
        }
    }
}

fn to_srgb8(srgb: Srgb) -> [u8; 3] {
    let color: palette::Srgb<u8> = srgb.into_format();
    [color.red, color.green, color.blue]
}

fn format_hex(srgb: Srgb) -> String {
    let [red, green, blue] = to_srgb8(srgb);
    format!("#{red:02X}{green:02X}{blue:02X}")
}

fn format_cmyk(srgb: Srgb) -> String {
    let [red, green, blue] = to_srgb8(srgb);
    let red = f32::from(red) / 255.0;
    let green = f32::from(green) / 255.0;
    let blue = f32::from(blue) / 255.0;

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
        "cmyk({:.0}%, {:.0}%, {:.0}%, {:.0}%)",
        cyan * 100.0,
        magenta * 100.0,
        yellow * 100.0,
        black * 100.0
    )
}

#[cfg(test)]
mod tests {
    use super::{ColorFormat, format_cmyk, format_hex};
    use crate::color::okhsl::Okhsl;
    use palette::Srgb;

    #[test]
    fn hex_uses_uppercase_pairs() {
        assert_eq!(format_hex(Srgb::new(1.0, 0.0, 0.0)), "#FF0000");
        assert_eq!(format_hex(Srgb::new(1.0, 0.5, 0.0)), "#FF8000");
    }

    #[test]
    fn cmyk_maps_red_and_black() {
        assert_eq!(
            format_cmyk(Srgb::new(1.0, 0.0, 0.0)),
            "cmyk(0%, 100%, 100%, 0%)"
        );
        assert_eq!(
            format_cmyk(Srgb::new(0.0, 0.0, 0.0)),
            "cmyk(0%, 0%, 0%, 100%)"
        );
    }

    #[test]
    fn okhsl_format_is_native() {
        let color = Okhsl::new(30.0, 0.5, 0.5);
        assert_eq!(
            ColorFormat::Okhsl.format(color),
            "okhsl(30.0, 0.500, 0.500)"
        );
    }

    fn component(text: &str, index: usize) -> f32 {
        text.split(['(', ','])
            .nth(index)
            .unwrap()
            .trim_end_matches([')', '%'])
            .trim()
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
            let hsl = ColorFormat::Hsl.format(color);
            assert!((0.0..360.0).contains(&component(&hsl, 1)), "{hsl}");
            let oklch = ColorFormat::Oklch.format(color);
            assert!((0.0..360.0).contains(&component(&oklch, 3)), "{oklch}");
        }
    }
}
