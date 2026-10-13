use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::config::APP_NAME;
use crate::persistence;

pub const STATE_FILE: &str = "state.toml";

/// Data the app changes as it is used, kept out of the config file so that
/// picking a colour never rewrites the user's settings.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct State {
    pub history: Vec<[u8; 3]>,
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
    use super::{STATE_FILE, State};

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
            history: vec![[1, 2, 3], [250, 128, 0]],
        };

        state.save_to(&path).unwrap();

        assert_eq!(State::load_from(&path), state);
        assert_eq!(
            State::load_from(&directory.join("absent.toml")),
            State::default()
        );

        std::fs::remove_dir_all(&directory).unwrap();
    }
}
