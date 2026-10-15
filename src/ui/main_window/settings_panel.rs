use eframe::egui;

use crate::color::{ColorFormat, Precision};
use crate::config::{Config, LaunchMode};
use crate::ui::widgets;

const SETTINGS_PANEL_ID: &str = "colorpickle-settings";
const SETTINGS_PANEL_MARGIN_X: i8 = 16;
pub(super) const MARGIN_Y: i8 = 10;

pub(super) struct Outcome {
    pub config_changed: bool,
    pub about_opened: bool,
}

pub(super) fn show(
    ui: &mut egui::Ui,
    config: &mut Config,
    background: egui::Color32,
    foreground: egui::Color32,
) -> Outcome {
    let mut outcome = Outcome {
        config_changed: false,
        about_opened: false,
    };
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
                                outcome.config_changed = true;
                            }
                        }
                    })
                    .response
                    .on_hover_text("Launch mode: open the main window, or start the screen picker.");
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
                                outcome.config_changed = true;
                            }
                        }
                    })
                    .response
                    .on_hover_text("Default format: shown in the input field and copied to clipboard.");
                egui::ComboBox::from_id_salt("precision")
                    .selected_text(config.precision.label())
                    .show_ui(ui, |ui| {
                        for precision in Precision::ALL {
                            if ui
                                .selectable_value(
                                    &mut config.precision,
                                    precision,
                                    precision.label(),
                                )
                                .changed()
                            {
                                outcome.config_changed = true;
                            }
                        }
                    })
                    .response
                    .on_hover_text(
                        "Precision: round values for general use, or keep decimals for accuracy.",
                    );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    outcome.about_opened = widgets::about_icon(ui);
                });
            });
        });
    outcome
}

fn launch_mode_label(mode: LaunchMode) -> &'static str {
    match mode {
        LaunchMode::UiFirst => "UI first",
        LaunchMode::PickerFirst => "Picker first",
    }
}
