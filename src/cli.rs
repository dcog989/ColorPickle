use clap::Parser;

use crate::config::{LaunchMode, LogLevel};

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

    /// Verbosity of the log file and stderr; overrides the config file's `log_level`.
    #[arg(long, value_enum)]
    pub log_level: Option<LogLevel>,
}
