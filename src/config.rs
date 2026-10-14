use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::color::okhsl::Okhsl;
use crate::color::{ColorFormat, Precision};
use crate::persistence;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Info => "info",
            Self::Debug => "debug",
            Self::Trace => "trace",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub launch_mode: LaunchMode,
    pub default_format: ColorFormat,
    pub precision: Precision,
    pub theme: Theme,
    pub log_level: LogLevel,
}

impl Config {
    pub fn format(&self, color: Okhsl) -> String {
        self.default_format.format(color, self.precision)
    }

    /// `$XDG_CONFIG_HOME/colorpickle/config.toml`, conventionally
    /// `~/.config/colorpickle/config.toml`.
    pub fn path() -> Option<PathBuf> {
        let dirs = directories::BaseDirs::new()?;
        Some(dirs.config_dir().join(APP_NAME).join(CONFIG_FILE))
    }

    pub fn load() -> Self {
        let Some(path) = Self::path() else {
            tracing::warn!("no config directory available; using defaults");
            return Self::default();
        };
        Self::load_from(&path)
    }

    fn load_from(path: &Path) -> Self {
        persistence::load(path)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path().context("no config directory available")?;
        self.save_to(&path)
    }

    fn save_to(&self, path: &Path) -> Result<()> {
        persistence::save(path, self)
    }
}

#[cfg(test)]
mod tests {
    use super::{CONFIG_FILE, Config, LaunchMode, LogLevel, Theme};
    use crate::color::{ColorFormat, Precision};

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
            precision: Precision::Precise,
            theme: Theme::Dark,
            log_level: LogLevel::Debug,
        };

        config.save_to(&path).unwrap();

        assert_eq!(Config::load_from(&path), config);
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("log_level = \"debug\""), "{text}");
        assert_eq!(
            Config::load_from(&directory.join("absent.toml")),
            Config::default()
        );

        std::fs::remove_dir_all(&directory).unwrap();
    }
}
