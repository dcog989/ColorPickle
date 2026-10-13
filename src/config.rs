use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::color::ColorFormat;
use crate::color::okhsl::Okhsl;

pub const APP_NAME: &str = "colorpickle";
pub const CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LaunchMode {
    #[default]
    #[value(name = "ui_first")]
    UiFirst,
    #[value(name = "picker_first")]
    PickerFirst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub launch_mode: LaunchMode,
    pub default_format: ColorFormat,
    pub theme: Theme,
    pub history: Vec<[u8; 3]>,
}

impl Config {
    pub fn format(&self, color: Okhsl) -> String {
        self.default_format.format(color)
    }

    pub fn path() -> Option<PathBuf> {
        directories::ProjectDirs::from("", "", APP_NAME)
            .map(|dirs| dirs.config_dir().join(CONFIG_FILE))
    }

    pub fn load() -> Self {
        let Some(path) = Self::path() else {
            tracing::warn!("no config directory available; using defaults");
            return Self::default();
        };
        Self::load_from(&path)
    }

    fn load_from(path: &Path) -> Self {
        if !path.exists() {
            return Self::default();
        }
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) => {
                tracing::warn!(
                    ?error,
                    path = %path.display(),
                    "failed to read config; using defaults"
                );
                return Self::default();
            }
        };
        match toml::from_str(&text) {
            Ok(config) => config,
            Err(error) => {
                tracing::warn!(
                    ?error,
                    path = %path.display(),
                    "failed to parse config; using defaults"
                );
                Self::default()
            }
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path().context("no config directory available")?;
        self.save_to(&path)
    }

    fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        let text = toml::to_string_pretty(self).context("failed to serialize config")?;
        let temporary = path.with_file_name(format!("{CONFIG_FILE}.tmp"));
        std::fs::write(&temporary, text)
            .with_context(|| format!("failed to write {}", temporary.display()))?;
        std::fs::rename(&temporary, path)
            .with_context(|| format!("failed to replace config at {}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::{CONFIG_FILE, Config, LaunchMode, Theme};
    use crate::color::ColorFormat;

    fn temporary_directory(name: &str) -> std::path::PathBuf {
        let directory =
            std::env::temp_dir().join(format!("colorpickle-test-{}-{name}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        directory
    }

    #[test]
    fn saves_and_loads_a_round_trip() {
        let directory = temporary_directory("round-trip");
        let path = directory.join(CONFIG_FILE);
        let config = Config {
            launch_mode: LaunchMode::PickerFirst,
            default_format: ColorFormat::Oklch,
            theme: Theme::Dark,
            history: vec![[1, 2, 3], [250, 128, 0]],
        };

        config.save_to(&path).unwrap();

        assert_eq!(Config::load_from(&path), config);
        assert!(!path.with_file_name(format!("{CONFIG_FILE}.tmp")).exists());
        assert_eq!(
            Config::load_from(&directory.join("absent.toml")),
            Config::default()
        );

        std::fs::remove_dir_all(&directory).unwrap();
    }
}
