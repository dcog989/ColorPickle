use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use palette::Srgb;
use serde::{Deserialize, Serialize};

use crate::color::okhsl::Okhsl;
use crate::config::APP_NAME;
use crate::persistence;

pub const STATE_FILE: &str = "state.toml";

const SRGB8_MAX: f32 = 255.0;

/// A persisted history colour, kept at full precision as Okhsl
/// `[hue, saturation, lightness]`. Older state files stored 8-bit sRGB, which
/// is still accepted on load and converted.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct StoredColor(pub [f32; 3]);

impl StoredColor {
    pub fn from_okhsl(color: Okhsl) -> Self {
        Self([color.hue(), color.saturation(), color.lightness()])
    }

    pub fn to_okhsl(self) -> Okhsl {
        Okhsl::new(self.0[0], self.0[1], self.0[2])
    }

    fn from_srgb8(red: u8, green: u8, blue: u8) -> Self {
        Self::from_okhsl(Okhsl::from_srgb(Srgb::new(
            f32::from(red) / SRGB8_MAX,
            f32::from(green) / SRGB8_MAX,
            f32::from(blue) / SRGB8_MAX,
        )))
    }
}

impl<'de> Deserialize<'de> for StoredColor {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Entry {
            Bytes([u8; 3]),
            Precise([f32; 3]),
        }

        Ok(match Entry::deserialize(deserializer)? {
            Entry::Bytes([red, green, blue]) => Self::from_srgb8(red, green, blue),
            Entry::Precise(components) => Self(components),
        })
    }
}

/// Data the app changes as it is used, kept out of the config file so that
/// picking a colour never rewrites the user's settings.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct State {
    pub history: Vec<StoredColor>,
}

/// `$XDG_STATE_HOME/colorpickle`, conventionally `~/.local/state/colorpickle`.
pub fn directory() -> Option<PathBuf> {
    let dirs = directories::BaseDirs::new()?;
    Some(dirs.state_dir()?.join(APP_NAME))
}

impl State {
    pub fn path() -> Option<PathBuf> {
        Some(directory()?.join(STATE_FILE))
    }

    pub fn load() -> Self {
        let Some(path) = Self::path() else {
            tracing::warn!("no state directory available; using defaults");
            return Self::default();
        };
        Self::load_from(&path)
    }

    fn load_from(path: &Path) -> Self {
        persistence::load(path)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path().context("no state directory available")?;
        self.save_to(&path)
    }

    fn save_to(&self, path: &Path) -> Result<()> {
        persistence::save(path, self)
    }
}

#[cfg(test)]
mod tests {
    use super::{STATE_FILE, State, StoredColor};

    fn temporary_directory(name: &str) -> std::path::PathBuf {
        let directory =
            std::env::temp_dir().join(format!("colorpickle-state-{}-{name}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        directory
    }

    #[test]
    fn saves_and_loads_a_round_trip() {
        let directory = temporary_directory("round-trip");
        let path = directory.join(STATE_FILE);
        let state = State {
            history: vec![
                StoredColor([336.34, 0.287, 0.4375]),
                StoredColor([0.0, 0.0, 0.0]),
            ],
        };

        state.save_to(&path).unwrap();

        assert_eq!(State::load_from(&path), state);
        assert_eq!(
            State::load_from(&directory.join("absent.toml")),
            State::default()
        );

        std::fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn loads_the_legacy_srgb8_history() {
        let directory = temporary_directory("legacy");
        let path = directory.join(STATE_FILE);
        std::fs::write(&path, "history = [[255, 0, 0], [0, 0, 0]]\n").unwrap();

        let state = State::load_from(&path);

        assert_eq!(state.history.len(), 2);
        assert_eq!(state.history[0].to_okhsl().to_srgb8(), [255, 0, 0]);
        assert_eq!(state.history[1].to_okhsl().to_srgb8(), [0, 0, 0]);

        std::fs::remove_dir_all(&directory).unwrap();
    }
}
