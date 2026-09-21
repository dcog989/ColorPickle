use eframe::egui;

use crate::color::ColorFormat;
use crate::config::{Config, LaunchMode};
use crate::ui::widgets;

const SETTINGS_PANEL_ID: &str = "colorpickle-settings";
const SETTINGS_PANEL_MARGIN_X: i8 = 16;
pub(super) const MARGIN_Y: i8 = 10;

pub(super) fn show(
    ui: &mut egui::Ui,
    config: &mut Config,
    background: egui::Color32,
    foreground: egui::Color32,
) -> bool {
    let mut config_changed = false;
    egui::Panel::bottom(SETTINGS_PANEL_ID)
        .resizable(false)
        .frame(
            egui::Frame::NONE
                .fill(background)
                .inner_margin(egui::Margin::symmetric(SETTINGS_PANEL_MARGIN_X, MARGIN_Y)),
        )
        .show(ui, |ui| {
            let row_height = widgets::row_height(ui);
            ui.spacing_mut().interact_size.y = row_height;
            ui.horizontal(|ui| {
                widgets::settings_icon(ui, foreground);
                egui::ComboBox::from_id_salt("launch-mode")
                    .selected_text(launch_mode_label(config.launch_mode))
                    .show_ui(ui, |ui| {
                        for mode in [LaunchMode::UiFirst, LaunchMode::PickerFirst] {
                            if ui
                                .selectable_value(
                                    &mut config.launch_mode,
                                    mode,
                                    launch_mode_label(mode),
                                )
                                .changed()
                            {
                                config_changed = true;
                            }
                        }
                    })
                    .response
                    .on_hover_text("Launch mode: open the main window, or start in the picker");
                egui::ComboBox::from_id_salt("default-format")
                    .selected_text(config.default_format.label())
                    .show_ui(ui, |ui| {
                        for format in ColorFormat::ALL {
                            if ui
                                .selectable_value(
                                    &mut config.default_format,
                                    format,
                                    format.label(),
                                )
                                .changed()
                            {
                                config_changed = true;
                            }
                        }
                    })
                    .response
                    .on_hover_text("Default format: shown in the input field and copied on pick");
            });
        });
    config_changed
}

fn launch_mode_label(mode: LaunchMode) -> &'static str {
    match mode {
        LaunchMode::UiFirst => "UI first",
        LaunchMode::PickerFirst => "Picker first",
    }
}
