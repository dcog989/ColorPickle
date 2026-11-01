use crate::color::okhsl::Okhsl;
use crate::state::StoredColor;

const HISTORY_LIMIT: usize = 8;

#[derive(Default)]
pub(super) struct History {
    colors: Vec<Okhsl>,
    /// What the file is known to hold, so a merge can tell this window's new
    /// picks from colours another instance wrote.
    saved: Vec<Okhsl>,
}

impl History {
    pub(super) fn from_stored(entries: &[StoredColor]) -> Self {
        let mut colors: Vec<Okhsl> = entries.iter().map(|entry| entry.to_okhsl()).collect();
        colors.truncate(HISTORY_LIMIT);
        Self {
            saved: colors.clone(),
            colors,
        }
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

    /// Merges the file another instance may have written. Picks this window
    /// made since the last save are the newest, so they stay in front; then
    /// come the file's entries in file order, so a newer pick from the other
    /// instance still lands before the older ones. A colour this window showed
    /// at the last save and no longer does was removed here, so the file must
    /// not bring it back.
    pub(super) fn merge_missing(&mut self, entries: &[StoredColor]) {
        let on_disk: Vec<Okhsl> = entries
            .iter()
            .take(HISTORY_LIMIT)
            .map(|entry| entry.to_okhsl())
            .collect();
        let removed: Vec<Okhsl> = self
            .saved
            .iter()
            .copied()
            .filter(|color| !self.colors.contains(color))
            .collect();
        let mut merged: Vec<Okhsl> = self
            .colors
            .iter()
            .copied()
            .filter(|color| !self.saved.contains(color))
            .collect();
        for color in &on_disk {
            if !merged.contains(color) && !removed.contains(color) {
                merged.push(*color);
            }
        }
        merged.truncate(HISTORY_LIMIT);
        self.colors = merged;
        self.saved = on_disk;
    }

    /// Call after the history has been written, so the next merge starts from it.
    pub(super) fn mark_saved(&mut self) {
        self.saved = self.colors.clone();
    }

    pub(super) fn colors(&self) -> &[Okhsl] {
        &self.colors
    }
}

#[cfg(test)]
mod tests {
    use super::{HISTORY_LIMIT, History};
    use crate::color::okhsl::Okhsl;
    use crate::state::StoredColor;

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

    #[test]
    fn merges_entries_another_instance_wrote() {
        let mut history = History::default();
        let colors = distinct_colors(3);
        let (old, mine, theirs) = (colors[0], colors[1], colors[2]);
        history.push(mine);

        let on_disk = [
            StoredColor::from_okhsl(theirs),
            StoredColor::from_okhsl(mine),
            StoredColor::from_okhsl(old),
        ];
        history.merge_missing(&on_disk);

        assert_eq!(history.colors(), [mine, theirs, old].as_slice());
    }

    #[test]
    fn does_not_restore_a_colour_removed_here() {
        let colors = distinct_colors(3);
        let stored: Vec<StoredColor> = colors.iter().map(|&c| StoredColor::from_okhsl(c)).collect();
        let mut history = History::from_stored(&stored);
        history.remove(colors[1]);
        history.merge_missing(&stored);
        assert_eq!(history.colors(), [colors[0], colors[2]].as_slice());

        history.clear();
        history.merge_missing(&stored);
        assert!(history.colors().is_empty());
    }

    #[test]
    fn keeps_a_pick_this_window_made_after_load_in_front() {
        let colors = distinct_colors(3);
        let (old, theirs, mine) = (colors[0], colors[1], colors[2]);
        let mut history = History::from_stored(&[StoredColor::from_okhsl(old)]);
        history.push(mine);

        let on_disk = [
            StoredColor::from_okhsl(theirs),
            StoredColor::from_okhsl(old),
        ];
        history.merge_missing(&on_disk);

        assert_eq!(history.colors(), [mine, theirs, old].as_slice());
    }
}
