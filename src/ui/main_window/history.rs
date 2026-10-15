use crate::color::okhsl::Okhsl;
use crate::state::StoredColor;

const HISTORY_LIMIT: usize = 8;

#[derive(Default)]
pub(super) struct History {
    colors: Vec<Okhsl>,
}

impl History {
    pub(super) fn from_stored(entries: &[StoredColor]) -> Self {
        let mut colors: Vec<Okhsl> = entries.iter().map(|entry| entry.to_okhsl()).collect();
        colors.truncate(HISTORY_LIMIT);
        Self { colors }
    }

    pub(super) fn to_stored(&self) -> Vec<StoredColor> {
        self.colors
            .iter()
            .copied()
            .map(StoredColor::from_okhsl)
            .collect()
    }

    pub(super) fn push(&mut self, color: Okhsl) {
        self.colors.retain(|existing| *existing != color);
        self.colors.insert(0, color);
        self.colors.truncate(HISTORY_LIMIT);
    }

    pub(super) fn clear(&mut self) {
        self.colors.clear();
    }

    pub(super) fn remove(&mut self, color: Okhsl) {
        self.colors.retain(|existing| *existing != color);
    }

    pub(super) fn colors(&self) -> &[Okhsl] {
        &self.colors
    }
}

#[cfg(test)]
mod tests {
    use super::{HISTORY_LIMIT, History};
    use crate::color::okhsl::Okhsl;

    const HUE_STEP: f32 = 10.0;

    fn distinct_colors(count: u32) -> Vec<Okhsl> {
        (0..count)
            .map(|step| Okhsl::new(step as f32 * HUE_STEP, 0.8, 0.5))
            .collect()
    }

    #[test]
    fn keeps_one_entry_per_colour_newest_first() {
        let mut history = History::default();
        let colors = distinct_colors(2);
        let (first, second) = (colors[0], colors[1]);

        history.push(first);
        history.push(second);
        history.push(first);

        assert_eq!(history.colors(), [first, second].as_slice());
    }

    #[test]
    fn caps_at_the_history_limit() {
        let mut history = History::default();
        let pushed = distinct_colors(HISTORY_LIMIT as u32 + 1);

        for color in &pushed {
            history.push(*color);
        }

        let expected: Vec<Okhsl> = pushed.iter().rev().take(HISTORY_LIMIT).copied().collect();
        assert_eq!(history.colors(), expected.as_slice());
    }
}
