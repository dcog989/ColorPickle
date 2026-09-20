pub mod main_window;
pub mod overlay;
pub mod picker;
pub mod slider;
pub mod theme;
pub mod widgets;

use anyhow::Result;
use eframe::egui;

use crate::color::okhsl::Okhsl;
use crate::config::Config;
use crate::ui::main_window::MainWindow;
use crate::ui::overlay::PickOutcome;

pub const WINDOW_TITLE: &str = "ColorPickle";
pub const WINDOW_WIDTH: f32 = 760.0;
pub const WINDOW_HEIGHT: f32 = 600.0;

pub fn run_ui(config: Config) -> Result<()> {
    run_main(config, None)
}

pub fn run_picker(config: Config) -> Result<()> {
    let outcome = overlay::run(config.clone())?;
    match outcome {
        Some(PickOutcome::Picked(color)) => run_main(config, Some(color)),
        _ => Ok(()),
    }
}

fn run_main(config: Config, initial: Option<Okhsl>) -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(WINDOW_TITLE)
            .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT]),
        ..Default::default()
    };

    eframe::run_native(
        WINDOW_TITLE,
        options,
        Box::new(move |_cc| {
            let window = match initial {
                Some(color) => MainWindow::new(config).with_initial(color),
                None => MainWindow::new(config),
            };
            Ok(Box::new(window))
        }),
    )
    .map_err(|error| anyhow::anyhow!("failed to run the UI: {error}"))
}
