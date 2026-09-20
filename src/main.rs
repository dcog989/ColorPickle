mod capture;
mod cli;
mod clipboard;
mod color;
mod config;
mod ui;

use anyhow::Result;
use clap::Parser;

use crate::cli::{Cli, LaunchMode};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = config::Config::load()?;
    let launch_mode = cli.launch_mode.unwrap_or(config.launch_mode);

    tracing::info!(?launch_mode, "starting ColorPickle");

    match launch_mode {
        LaunchMode::UiFirst => ui::run_ui(config),
        LaunchMode::PickerFirst => ui::run_picker(config),
    }
}
