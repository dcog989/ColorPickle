pub mod icons;
pub mod main_window;
pub mod overlay;
pub mod picker;
pub mod pixels;
pub mod slider;
pub mod theme;
pub mod widgets;

use anyhow::Result;
use eframe::egui;

use crate::config::Config;
use crate::ui::main_window::MainWindow;

pub const WINDOW_TITLE: &str = "ColorPickle";
pub const WINDOW_WIDTH: f32 = 760.0;
pub const WINDOW_HEIGHT: f32 = 600.0;

pub fn run_ui(config: Config) -> Result<()> {
    run_main(config, false)
}

pub fn run_picker(config: Config) -> Result<()> {
    run_main(config, true)
}

fn run_main(config: Config, picker_first: bool) -> Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_title(WINDOW_TITLE)
        .with_app_id("colorpickle")
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
            icons::install(&cc.egui_ctx);
            let mut window = if picker_first {
                MainWindow::new(config).picker_first()
            } else {
                MainWindow::new(config)
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
