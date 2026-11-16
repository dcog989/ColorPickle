pub mod frame_stats;
pub mod icons;
pub mod main_window;
pub mod overlay;
pub mod picker;
pub mod pixels;
pub mod slider;
pub mod theme;
pub mod widgets;

use std::process::Command;

use anyhow::{Context, Result};
use eframe::egui;

use crate::color::okhsl::Okhsl;
use crate::config::Config;
use crate::logging::Logging;
use crate::state::State;
use crate::ui::main_window::MainWindow;
use crate::ui::overlay::PickOutcome;

pub const WINDOW_TITLE: &str = "ColorPickle";
/// Matches the `.desktop` file name so a Wayland compositor can find the icon.
pub const APP_ID: &str = "colorpickle";
pub const WINDOW_WIDTH: f32 = 760.0;
pub const WINDOW_HEIGHT: f32 = 600.0;
pub const ICON_PNG: &[u8] = include_bytes!("../../assets/colorpickle.png");

/// Carries the colour from a picker-first process to the main window it opens.
const INITIAL_COLOR_VARIABLE: &str = "COLORPICKLE_INITIAL_COLOR";
const UI_FIRST: &str = "ui_first";

pub fn run_ui(config: Config, state: State, logging: Logging) -> Result<()> {
    run_main(config, state, logging, initial_color())
}

/// Captures before any window exists, so a hotkey launch shows only the picker.
pub fn run_picker(config: Config) -> Result<()> {
    match overlay::run(config)? {
        Some(PickOutcome::Picked(color)) => open_main_window(color),
        _ => Ok(()),
    }
}

/// A second `eframe::run_native` in this process dies in winit's event loop once
/// the Wayland clipboard has used the connection, so the main window is a fresh
/// process instead. This one waits for it so the desktop's systemd scope does
/// not kill the child when the picker exits.
fn open_main_window(color: Okhsl) -> Result<()> {
    let executable = std::env::current_exe().context("failed to find the executable")?;
    let mut child = Command::new(executable)
        .args(["--launch-mode", UI_FIRST])
        .env(INITIAL_COLOR_VARIABLE, format_initial_color(color))
        .spawn()
        .context("failed to open the main window")?;
    let status = child.wait().context("failed to wait for the main window")?;
    if !status.success() {
        anyhow::bail!("the main window exited with {status}");
    }
    Ok(())
}

fn format_initial_color(color: Okhsl) -> String {
    // Display prints the shortest text that parses back to the same f32.
    format!(
        "{} {} {}",
        color.hue(),
        color.saturation(),
        color.lightness()
    )
}

fn initial_color() -> Option<Okhsl> {
    parse_initial_color(&std::env::var(INITIAL_COLOR_VARIABLE).ok()?)
}

fn parse_initial_color(text: &str) -> Option<Okhsl> {
    let mut parts = text.split_whitespace();
    let hue = parts.next()?.parse().ok()?;
    let saturation = parts.next()?.parse().ok()?;
    let lightness = parts.next()?.parse().ok()?;
    Some(Okhsl::new(hue, saturation, lightness))
}

fn run_main(config: Config, state: State, logging: Logging, initial: Option<Okhsl>) -> Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_title(WINDOW_TITLE)
        .with_app_id(APP_ID)
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
            let mut window = match initial {
                Some(color) => MainWindow::new(config, state, logging).with_initial(color),
                None => MainWindow::new(config, state, logging),
            };
            window.install_theme(&cc.egui_ctx);
            Ok(Box::new(window))
        }),
    )
    .map_err(|error| anyhow::anyhow!("failed to run the UI: {error}"))
}

fn app_icon() -> Option<egui::IconData> {
    match image::load_from_memory(ICON_PNG) {
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

#[cfg(test)]
mod tests {
    use super::{format_initial_color, parse_initial_color};
    use crate::color::okhsl::Okhsl;

    #[test]
    fn the_picked_colour_survives_the_handoff() {
        let color = Okhsl::new(202.584_05, 0.938_364_8, 0.670_973);
        assert_eq!(
            parse_initial_color(&format_initial_color(color)),
            Some(color)
        );
        assert_eq!(parse_initial_color("not a colour"), None);
    }
}
