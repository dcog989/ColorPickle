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
    let mut viewport = egui::ViewportBuilder::default()
        .with_title(WINDOW_TITLE)
        .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT]);
    if let Some(icon) = app_icon() {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        WINDOW_TITLE,
        options,
        Box::new(move |cc| {
            let mut window = match initial {
                Some(color) => MainWindow::new(config).with_initial(color),
                None => MainWindow::new(config),
            };
            window.apply_theme(&cc.egui_ctx);
            Ok(Box::new(window))
        }),
    )
    .map_err(|error| anyhow::anyhow!("failed to run the UI: {error}"))
}

fn app_icon() -> Option<egui::IconData> {
    let bytes: &[u8] = include_bytes!("../../assets/colorpickle.png");
    match image::load_from_memory(bytes) {
        Ok(image) => {
            let image = image.into_rgba8();
            let width = image.width();
            let height = image.height();
            Some(egui::IconData {
                rgba: image.into_raw(),
                width,
                height,
            })
        }
        Err(error) => {
            tracing::warn!(?error, "failed to decode the app icon");
            None
        }
    }
}
