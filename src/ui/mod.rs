pub mod main_window;
pub mod overlay;
pub mod picker;
pub mod slider;
pub mod theme;
pub mod widgets;

use anyhow::Result;
use eframe::egui;

use crate::config::Config;
use crate::ui::main_window::MainWindow;

pub const WINDOW_TITLE: &str = "ColorPickle";
pub const WINDOW_WIDTH: f32 = 600.0;
pub const WINDOW_HEIGHT: f32 = 600.0;

pub fn run_ui(config: Config) -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(WINDOW_TITLE)
            .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT]),
        ..Default::default()
    };

    eframe::run_native(
        WINDOW_TITLE,
        options,
        Box::new(move |_cc| Ok(Box::new(MainWindow::new(config)))),
    )
    .map_err(|error| anyhow::anyhow!("failed to run the UI: {error}"))
}

pub fn run_picker(config: Config) -> Result<()> {
    overlay::run(config)
}
