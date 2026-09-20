use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LaunchMode {
    #[default]
    #[value(name = "ui_first")]
    UiFirst,
    #[value(name = "picker_first")]
    PickerFirst,
}

#[derive(Debug, Parser)]
#[command(
    name = "colorpickle",
    version,
    about = "Pick a color from anywhere on screen."
)]
pub struct Cli {
    /// Overrides the launch mode stored in the config file.
    #[arg(long, value_enum)]
    pub launch_mode: Option<LaunchMode>,
}
