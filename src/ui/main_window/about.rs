use std::path::PathBuf;

use eframe::egui;

use crate::config::Config;
use crate::logging;
use crate::state::State;
use crate::ui::{ICON_PNG, WINDOW_TITLE};

const WINDOW_ID: &str = "colorpickle-about";
const LOGO_SIZE: f32 = 144.0;
const LOGO_SPACING: f32 = 12.0;
const SECTION_SPACING: f32 = 14.0;
const ROW_SPACING: f32 = 14.0;
const NAME_FONT_SIZE: f32 = 22.0;
const PATH_FONT_SIZE: f32 = 13.0;
const WINDOW_WIDTH: f32 = 480.0;

#[derive(Default)]
pub struct About {
    open: bool,
    logo: Option<egui::TextureHandle>,
}

impl About {
    pub fn open(&mut self) {
        self.open = true;
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.open {
            return;
        }
        if self.logo.is_none() {
            self.logo = logo_texture(ctx);
        }

        let mut open = self.open;
        egui::Window::new(format!("About {WINDOW_TITLE}"))
            .id(egui::Id::new(WINDOW_ID))
            .collapsible(false)
            .resizable(false)
            .default_width(WINDOW_WIDTH)
            .open(&mut open)
            .show(ctx, |ui| self.contents(ui));
        self.open = open;
    }

    fn contents(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            if let Some(logo) = &self.logo {
                ui.add(egui::Image::new(logo).fit_to_exact_size(egui::vec2(LOGO_SIZE, LOGO_SIZE)));
                ui.add_space(LOGO_SPACING);
            }
            ui.label(
                egui::RichText::new(WINDOW_TITLE)
                    .size(NAME_FONT_SIZE)
                    .strong(),
            );
            ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
        });

        ui.add_space(SECTION_SPACING);
        ui.separator();
        ui.add_space(SECTION_SPACING);

        for (label, path) in paths() {
            ui.label(egui::RichText::new(label).strong());
            path_value(ui, path);
            ui.add_space(ROW_SPACING);
        }
    }
}

fn paths() -> [(&'static str, Option<PathBuf>); 4] {
    [
        ("Config", Config::path()),
        ("History", State::path()),
        ("Logs", logging::directory()),
        ("Binary", std::env::current_exe().ok()),
    ]
}

fn path_value(ui: &mut egui::Ui, path: Option<PathBuf>) {
    let Some(path) = path else {
        ui.weak("unavailable");
        return;
    };

    let text = path.display().to_string();
    let response = ui.add(
        egui::Label::new(
            egui::RichText::new(text.clone())
                .monospace()
                .size(PATH_FONT_SIZE),
        )
        .truncate()
        .selectable(false)
        .sense(egui::Sense::click()),
    );
    if response.on_hover_text("Click to copy path").clicked() {
        ui.ctx().copy_text(text);
    }
}

fn logo_texture(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let image = image::load_from_memory(ICON_PNG).ok()?.into_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &image.into_raw());
    Some(ctx.load_texture(
        "colorpickle-logo",
        color_image,
        egui::TextureOptions::LINEAR,
    ))
}
