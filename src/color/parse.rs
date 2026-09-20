use palette::{FromColor, Hsl, Lab, Oklab, OklabHue, Oklch, RgbHue, Srgb};

use crate::color::okhsl::Okhsl;

const HEX_SHORTHAND_LEN: usize = 3;
const HEX_FULL_LEN: usize = 6;
const HEX_SHORTHAND_SCALE: u8 = 17;
const CHANNEL_MAX: f32 = 255.0;
const PERCENT: f32 = 100.0;

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
    u8::from_str_radix(text, 16).ok()
}

fn parse_function(name: &str, args: &str) -> Option<Okhsl> {
    let values = split_values(args);
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

fn parse_rgb(values: &[f32]) -> Option<Okhsl> {
    let [red, green, blue] = three(values)?;
    Some(from_srgb8(byte(red), byte(green), byte(blue)))
}

fn parse_hsl(values: &[f32]) -> Option<Okhsl> {
    let [hue, saturation, lightness] = three(values)?;
    let hsl = Hsl::new(
        RgbHue::from_degrees(hue),
        unit(saturation),
        unit(lightness),
    );
    Some(Okhsl::from_srgb(Srgb::from_color(hsl)))
}

fn parse_okhsl(values: &[f32]) -> Option<Okhsl> {
    let [hue, saturation, lightness] = three(values)?;
    Some(Okhsl::new(hue, unit(saturation), unit(lightness)))
}

fn parse_oklch(values: &[f32]) -> Option<Okhsl> {
    let [lightness, chroma, hue] = three(values)?;
    let oklch = Oklch::new(lightness, chroma, OklabHue::from_degrees(hue));
    Some(Okhsl::from_srgb(Srgb::from_color(oklch)))
}

fn parse_oklab(values: &[f32]) -> Option<Okhsl> {
    let [lightness, a, b] = three(values)?;
    Some(Okhsl::from_srgb(Srgb::from_color(Oklab::new(
        lightness, a, b,
    ))))
}

fn parse_cmyk(values: &[f32]) -> Option<Okhsl> {
    let [cyan, magenta, yellow, black] = four(values)?;
    let cyan = unit(cyan);
    let magenta = unit(magenta);
    let yellow = unit(yellow);
    let black = unit(black);
    Some(Okhsl::from_srgb(Srgb::new(
        (1.0 - cyan) * (1.0 - black),
        (1.0 - magenta) * (1.0 - black),
        (1.0 - yellow) * (1.0 - black),
    )))
}

fn parse_cielab(values: &[f32]) -> Option<Okhsl> {
    let [lightness, a, b] = three(values)?;
    let lab: Lab = Lab::new(lightness, a, b);
    Some(Okhsl::from_srgb(Srgb::from_color(lab)))
}

fn split_values(args: &str) -> Vec<f32> {
    args.split(|character: char| character == ',' || character.is_whitespace())
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.trim_end_matches('%').parse::<f32>().ok())
        .collect()
}

fn three(values: &[f32]) -> Option<[f32; 3]> {
    match values {
        [first, second, third] => Some([*first, *second, *third]),
        _ => None,
    }
}

fn four(values: &[f32]) -> Option<[f32; 4]> {
    match values {
        [first, second, third, fourth] => Some([*first, *second, *third, *fourth]),
        _ => None,
    }
}

fn unit(value: f32) -> f32 {
    let scaled = if value > 1.0 { value / PERCENT } else { value };
    scaled.clamp(0.0, 1.0)
}

fn byte(value: f32) -> u8 {
    value.round().clamp(0.0, CHANNEL_MAX) as u8
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
        assert_eq!(
            parse("rgb(0, 128, 255)").unwrap().to_srgb8(),
            [0, 128, 255]
        );
        let hue = parse("okhsl(210, 0.5, 0.5)").unwrap().hue();
        assert!((hue - 210.0).abs() < 1.0, "hue was {hue}");
    }

    #[test]
    fn rejects_invalid_input() {
        assert!(parse("not-a-color").is_none());
        assert!(parse("rgb(1, 2)").is_none());
        assert!(parse("#12345").is_none());
        assert!(parse("").is_none());
    }
}
