mod capture;
mod cli;
mod clipboard;
mod color;
mod config;
mod logging;
mod persistence;
mod state;
mod ui;

use anyhow::Result;
use clap::Parser;

use crate::cli::Cli;
use crate::config::LaunchMode;

fn main() -> Result<()> {
    let cli = Cli::parse();
    // Kept for the whole run: dropping the guard discards lines still queued for the log file.
    let _log_guard = logging::init(cli.log_level.unwrap_or_default());

    let config = config::Config::load();
    let state = state::State::load();
    let launch_mode = cli.launch_mode.unwrap_or(config.launch_mode);

    tracing::info!(?launch_mode, "starting ColorPickle");

    match launch_mode {
        LaunchMode::UiFirst => ui::run_ui(config, state),
        LaunchMode::PickerFirst => ui::run_picker(config, state),
    }
}
