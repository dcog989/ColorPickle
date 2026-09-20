use palette::{FromColor, Hsl, Lab, Oklab, OklabHue, Oklch, RgbHue, Srgb};

use crate::color::okhsl::Okhsl;

const HEX_SHORTHAND_LEN: usize = 3;
const HEX_FULL_LEN: usize = 6;
const HEX_SHORTHAND_SCALE: u8 = 17;
const CHANNEL_MAX: f32 = 255.0;
const PERCENT: f32 = 100.0;

#[derive(Clone, Copy)]
enum Value {
    Number(f32),
    Percent(f32),
}

impl Value {
    fn number(self) -> f32 {
        match self {
            Self::Number(value) => value,
            Self::Percent(value) => value / PERCENT,
        }
    }

    fn fraction(self) -> f32 {
        self.number().clamp(0.0, 1.0)
    }

    fn percentage(self) -> f32 {
        let value = match self {
            Self::Number(value) | Self::Percent(value) => value,
        };
        (value / PERCENT).clamp(0.0, 1.0)
    }

    fn byte(self) -> u8 {
        match self {
            Self::Number(value) => value,
            Self::Percent(value) => value / PERCENT * CHANNEL_MAX,
        }
        .round()
        .clamp(0.0, CHANNEL_MAX) as u8
    }
}

pub fn parse(input: &str) -> Option<Okhsl> {
    let text = input.trim().to_ascii_lowercase();
    if text.is_empty() {
        return None;
    }
    if let Some(hex) = text.strip_prefix('#') {
        return parse_hex(hex);
    }
    if let Some(open) = text.find('(') {
        let name = text[..open].trim();
        let args = text[open + 1..].strip_suffix(')')?;
        return parse_function(name, args);
    }
    palette::named::from_str(&text).map(|named| {
        let srgb: Srgb = named.into_format();
        Okhsl::from_srgb(srgb)
    })
}

fn parse_hex(hex: &str) -> Option<Okhsl> {
    if !hex.is_ascii() {
        return None;
    }
    let (red, green, blue) = match hex.len() {
        HEX_SHORTHAND_LEN => {
            let red = hex_byte(&hex[0..1])? * HEX_SHORTHAND_SCALE;
            let green = hex_byte(&hex[1..2])? * HEX_SHORTHAND_SCALE;
            let blue = hex_byte(&hex[2..3])? * HEX_SHORTHAND_SCALE;
            (red, green, blue)
        }
        HEX_FULL_LEN => (
            hex_byte(&hex[0..2])?,
            hex_byte(&hex[2..4])?,
            hex_byte(&hex[4..6])?,
        ),
        _ => return None,
    };
    Some(from_srgb8(red, green, blue))
}

fn hex_byte(text: &str) -> Option<u8> {
    if !text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    u8::from_str_radix(text, 16).ok()
}

fn parse_function(name: &str, args: &str) -> Option<Okhsl> {
    let values = split_values(args)?;
    match name {
        "rgb" => parse_rgb(&values),
        "hsl" => parse_hsl(&values),
        "okhsl" => parse_okhsl(&values),
        "oklch" => parse_oklch(&values),
        "oklab" => parse_oklab(&values),
        "cmyk" => parse_cmyk(&values),
        "lab" => parse_cielab(&values),
        _ => None,
    }
}

fn parse_rgb(values: &[Value]) -> Option<Okhsl> {
    let [red, green, blue] = three(values)?;
    Some(from_srgb8(red.byte(), green.byte(), blue.byte()))
}

fn parse_hsl(values: &[Value]) -> Option<Okhsl> {
    let [hue, saturation, lightness] = three(values)?;
    let hsl = Hsl::new(
        RgbHue::from_degrees(hue.number()),
        saturation.percentage(),
        lightness.percentage(),
    );
    Some(Okhsl::from_srgb(Srgb::from_color(hsl)))
}

fn parse_okhsl(values: &[Value]) -> Option<Okhsl> {
    let [hue, saturation, lightness] = three(values)?;
    Some(Okhsl::new(
        hue.number(),
        saturation.fraction(),
        lightness.fraction(),
    ))
}

fn parse_oklch(values: &[Value]) -> Option<Okhsl> {
    let [lightness, chroma, hue] = three(values)?;
    let oklch = Oklch::new(
        lightness.number(),
        chroma.number(),
        OklabHue::from_degrees(hue.number()),
    );
    Some(Okhsl::from_srgb(Srgb::from_color(oklch)))
}

fn parse_oklab(values: &[Value]) -> Option<Okhsl> {
    let [lightness, a, b] = three(values)?;
    Some(Okhsl::from_srgb(Srgb::from_color(Oklab::new(
        lightness.number(),
        a.number(),
        b.number(),
    ))))
}

fn parse_cmyk(values: &[Value]) -> Option<Okhsl> {
    let [cyan, magenta, yellow, black] = four(values)?;
    let cyan = cyan.percentage();
    let magenta = magenta.percentage();
    let yellow = yellow.percentage();
    let black = black.percentage();
    Some(Okhsl::from_srgb(Srgb::new(
        (1.0 - cyan) * (1.0 - black),
        (1.0 - magenta) * (1.0 - black),
        (1.0 - yellow) * (1.0 - black),
    )))
}

fn parse_cielab(values: &[Value]) -> Option<Okhsl> {
    let [lightness, a, b] = three(values)?;
    let lab: Lab = Lab::new(lightness.number(), a.number(), b.number());
    Some(Okhsl::from_srgb(Srgb::from_color(lab)))
}

fn split_values(args: &str) -> Option<Vec<Value>> {
    args.split(|character: char| character == ',' || character.is_whitespace())
        .filter(|part| !part.is_empty())
        .map(|part| match part.strip_suffix('%') {
            Some(number) => number.parse::<f32>().ok().map(Value::Percent),
            None => part.parse::<f32>().ok().map(Value::Number),
        })
        .collect()
}

fn three(values: &[Value]) -> Option<[Value; 3]> {
    match values {
        [first, second, third] => Some([*first, *second, *third]),
        _ => None,
    }
}

fn four(values: &[Value]) -> Option<[Value; 4]> {
    match values {
        [first, second, third, fourth] => Some([*first, *second, *third, *fourth]),
        _ => None,
    }
}

fn from_srgb8(red: u8, green: u8, blue: u8) -> Okhsl {
    Okhsl::from_srgb(Srgb::new(
        f32::from(red) / CHANNEL_MAX,
        f32::from(green) / CHANNEL_MAX,
        f32::from(blue) / CHANNEL_MAX,
    ))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::color::ColorFormat;

    // 8-bit channels; HSL and CMYK quantise to whole degrees/percent, so a
    // near-lossless round trip is the most these formats can guarantee.
    const ROUND_TRIP_TOLERANCE: u8 = 6;

    #[test]
    fn hex_shorthand_matches_full() {
        assert_eq!(
            parse("#f00").unwrap().to_srgb8(),
            parse("#ff0000").unwrap().to_srgb8()
        );
    }

    #[test]
    fn parses_css_named_colors() {
        assert_eq!(parse("red").unwrap().to_srgb8(), [255, 0, 0]);
        assert_eq!(parse("Olive").unwrap().to_srgb8(), [128, 128, 0]);
    }

    #[test]
    fn parses_function_forms() {
        assert_eq!(parse("rgb(0, 128, 255)").unwrap().to_srgb8(), [0, 128, 255]);
        let hue = parse("okhsl(210, 0.5, 0.5)").unwrap().hue();
        assert!((hue - 210.0).abs() < 1.0, "hue was {hue}");
    }

    #[test]
    fn rejects_invalid_input() {
        assert!(parse("not-a-color").is_none());
        assert!(parse("rgb(1, 2)").is_none());
        assert!(parse("#12345").is_none());
        assert!(parse("#+f0000").is_none());
        assert!(parse("rgb(255, abc, 0)").is_none());
        assert!(parse("rgb(255, abc, 0, 0)").is_none());
        assert!(parse("").is_none());
    }

    #[test]
    fn percent_is_always_a_hundredth() {
        let black = parse("hsl(0, 0%, 1%)").unwrap().to_srgb8();
        assert!(black.iter().all(|channel| *channel <= 3), "{black:?}");

        let near_white = parse("cmyk(0%, 0%, 0%, 1%)").unwrap().to_srgb8();
        assert!(
            near_white.iter().all(|channel| *channel >= 252),
            "{near_white:?}"
        );
    }

    #[test]
    fn bare_numbers_use_format_scale() {
        assert_eq!(
            parse("hsl(0, 100, 100)").unwrap().to_srgb8(),
            parse("hsl(0, 100%, 100%)").unwrap().to_srgb8()
        );
        assert_eq!(
            parse("cmyk(0, 0, 0, 100)").unwrap().to_srgb8(),
            parse("cmyk(0%, 0%, 0%, 100%)").unwrap().to_srgb8()
        );
        assert_eq!(parse("rgb(255, 0, 0)").unwrap().to_srgb8(), [255, 0, 0]);
        assert!((parse("okhsl(0, 1, 0.5)").unwrap().saturation() - 1.0).abs() < 1e-3);
    }

    #[test]
    fn formats_round_trip() {
        for text in [
            "#ff0000", "#fcfcfc", "#0a0a0a", "#ffffff", "#000000", "#94a135", "#3f7fbf", "#7f3fbf",
        ] {
            let color = parse(text).unwrap();
            let expected = color.to_srgb8();
            for format in ColorFormat::ALL {
                let formatted = format.format(color);
                let parsed = parse(&formatted).unwrap();
                let actual = parsed.to_srgb8();
                for (channel, expected_channel) in actual.iter().zip(expected.iter()) {
                    assert!(
                        channel.abs_diff(*expected_channel) <= ROUND_TRIP_TOLERANCE,
                        "{format:?} round-tripped {text} {expected:?} via {formatted} to {actual:?}"
                    );
                }
            }
        }
    }
}
