use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::cli::LaunchMode;
use crate::color::ColorFormat;

pub const APP_NAME: &str = "colorpickle";
pub const CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub launch_mode: LaunchMode,
    pub default_format: ColorFormat,
    pub theme: Theme,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            launch_mode: LaunchMode::default(),
            default_format: ColorFormat::default(),
            theme: Theme::default(),
        }
    }
}

impl Config {
    pub fn path() -> Option<PathBuf> {
        directories::ProjectDirs::from("", "", APP_NAME)
            .map(|dirs| dirs.config_dir().join(CONFIG_FILE))
    }

    pub fn load() -> Self {
        let Some(path) = Self::path() else {
            tracing::warn!("no config directory available; using defaults");
            return Self::default();
        };
        if !path.exists() {
            return Self::default();
        }
        let text = match std::fs::read_to_string(&path) {
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
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        let text = toml::to_string_pretty(self).context("failed to serialize config")?;
        let temporary = path.with_file_name(format!("{CONFIG_FILE}.tmp"));
        std::fs::write(&temporary, text)
            .with_context(|| format!("failed to write {}", temporary.display()))?;
        std::fs::rename(&temporary, &path)
            .with_context(|| format!("failed to replace config at {}", path.display()))
    }
}
