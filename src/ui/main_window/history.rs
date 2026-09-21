use crate::color::okhsl::Okhsl;

const HISTORY_LIMIT: usize = 8;

#[derive(Default)]
pub(super) struct History {
    colors: Vec<Okhsl>,
}

impl History {
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
