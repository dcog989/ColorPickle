use palette::Srgb;

use crate::color::okhsl::Okhsl;

const HISTORY_LIMIT: usize = 8;
const CHANNEL_MAX: f32 = 255.0;

#[derive(Default)]
pub(super) struct History {
    colors: Vec<Okhsl>,
}

impl History {
    pub(super) fn from_srgb8(entries: &[[u8; 3]]) -> Self {
        let mut colors: Vec<Okhsl> = entries
            .iter()
            .map(|&[red, green, blue]| {
                Okhsl::from_srgb(Srgb::new(
                    f32::from(red) / CHANNEL_MAX,
                    f32::from(green) / CHANNEL_MAX,
                    f32::from(blue) / CHANNEL_MAX,
                ))
            })
            .collect();
        colors.truncate(HISTORY_LIMIT);
        Self { colors }
    }

    pub(super) fn to_srgb8(&self) -> Vec<[u8; 3]> {
        self.colors.iter().copied().map(Okhsl::to_srgb8).collect()
    }

    pub(super) fn push(&mut self, color: Okhsl) {
        self.colors
            .retain(|existing| existing.to_srgb8() != color.to_srgb8());
        self.colors.insert(0, color);
        self.colors.truncate(HISTORY_LIMIT);
    }

    pub(super) fn clear(&mut self) {
        self.colors.clear();
    }

    pub(super) fn colors(&self) -> &[Okhsl] {
        &self.colors
    }
}
