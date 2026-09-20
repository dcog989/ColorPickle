use eframe::egui;

use crate::clipboard;
use crate::color::ColorFormat;
use crate::color::okhsl::Okhsl;
use crate::config::Config;
use crate::ui::picker::{Event, PickerController};
use crate::ui::slider;
use crate::ui::theme::{self, color32, contrast_color32};

const SLIDER_WIDTH: f32 = 26.0;
const SLIDER_HEIGHT: f32 = 200.0;
const HUE_MAX_DEGREES: f32 = 360.0;
const HUE_FRACTION_MAX: f32 = 1.0 - f32::EPSILON;
const DEFAULT_HUE_DEGREES: f32 = 180.0;
const DEFAULT_SATURATION: f32 = 0.5;
const DEFAULT_LIGHTNESS: f32 = 0.5;
const EYEDROP_ICON_SIZE: f32 = 22.0;
const EYEDROP_OUTER_RADIUS: f32 = 8.0;
const EYEDROP_INNER_RADIUS: f32 = 2.5;
const EYEDROP_STROKE_WIDTH: f32 = 1.5;
const SATURATION_TOOLTIP: &str = "Saturation is perceptual (Okhsl-normalised), so its visual effect varies slightly with lightness.";

pub struct MainWindow {
    config: Config,
    color: Okhsl,
    status: Option<String>,
    picker: PickerController,
}

impl MainWindow {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            color: Okhsl::new(DEFAULT_HUE_DEGREES, DEFAULT_SATURATION, DEFAULT_LIGHTNESS),
            status: None,
            picker: PickerController::new(),
        }
    }

    fn copy(&mut self, format: ColorFormat) {
        let value = format.format(self.color);
        self.status = Some(match clipboard::set_text(value) {
            Ok(()) => format!("copied {}", format.label()),
            Err(error) => format!("copy failed: {error}"),
        });
    }

    fn open_picker(&mut self) {
        if self.picker.request() {
            self.status = Some("capturing screen...".to_owned());
        }
    }

    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Ready => self.status = Some("picking from screen...".to_owned()),
            Event::CaptureFailed(error) => {
                self.status = Some(format!("capture failed: {error}"));
            }
            Event::ThreadStopped => {
                self.status = Some("capture thread stopped unexpectedly".to_owned());
            }
            Event::Picked(color) => {
                self.color = color;
                let value = self.config.default_format.format(color);
                self.status = Some(match clipboard::set_text(value) {
                    Ok(()) => format!("picked {}", self.config.default_format.label()),
                    Err(error) => format!("copy failed: {error}"),
                });
            }
            Event::Dismissed => self.status = Some("picking cancelled".to_owned()),
        }
    }
}

impl eframe::App for MainWindow {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let mut open_picker = false;

        theme::apply(&ctx, self.config.theme, self.color);

        let panel_frame = egui::Frame::central_panel(ui.style()).fill(color32(self.color));
        egui::CentralPanel::default()
            .frame(panel_frame)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    open_picker |= draw_picker_launcher(ui, self.color);
                    ui.heading("ColorPickle");
                });

                let mut hue = self.color.hue() / HUE_MAX_DEGREES;
                let mut saturation = self.color.saturation();
                let mut lightness = self.color.lightness();
                let slider_size = egui::vec2(SLIDER_WIDTH, SLIDER_HEIGHT);

                ui.horizontal(|ui| {
                    let hue_saturation = saturation;
                    let hue_lightness = lightness;
                    slider::vertical(ui, slider_size, &mut hue, move |value| {
                        color32(Okhsl::new(
                            value * HUE_MAX_DEGREES,
                            hue_saturation,
                            hue_lightness,
                        ))
                    });

                    let saturation_hue = hue * HUE_MAX_DEGREES;
                    let saturation_lightness = lightness;
                    slider::vertical(ui, slider_size, &mut saturation, move |value| {
                        color32(Okhsl::new(saturation_hue, value, saturation_lightness))
                    })
                    .on_hover_text(SATURATION_TOOLTIP);

                    let lightness_hue = hue * HUE_MAX_DEGREES;
                    let lightness_saturation = saturation;
                    slider::vertical(ui, slider_size, &mut lightness, move |value| {
                        color32(Okhsl::new(lightness_hue, lightness_saturation, value))
                    });
                });

                self.color = Okhsl::new(
                    hue.min(HUE_FRACTION_MAX) * HUE_MAX_DEGREES,
                    saturation,
                    lightness,
                );

                ui.horizontal_wrapped(|ui| {
                    for format in ColorFormat::ALL {
                        let value = format.format(self.color);
                        let response = ui.button(format.label()).on_hover_text(value.as_str());
                        if response.clicked() {
                            self.copy(format);
                        }
                    }
                });

                if let Some(status) = &self.status {
                    ui.label(status.as_str());
                }
            });

        if open_picker {
            self.open_picker();
        }

        if let Some(event) = self.picker.update(&ctx, &self.config) {
            self.handle_event(event);
        }
    }
}

fn draw_picker_launcher(ui: &mut egui::Ui, color: Okhsl) -> bool {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(EYEDROP_ICON_SIZE, EYEDROP_ICON_SIZE),
        egui::Sense::click(),
    );
    let response = response
        .on_hover_cursor(egui::CursorIcon::Crosshair)
        .on_hover_text("Pick from screen");
    paint_crosshair(ui.painter(), rect, contrast_color32(color));
    response.clicked()
}

fn paint_crosshair(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    let center = rect.center();
    let stroke = egui::Stroke::new(EYEDROP_STROKE_WIDTH, color);
    painter.circle_stroke(center, EYEDROP_OUTER_RADIUS, stroke);
    painter.circle_filled(center, EYEDROP_INNER_RADIUS, color);
    painter.line_segment(
        [
            egui::pos2(center.x - EYEDROP_OUTER_RADIUS, center.y),
            egui::pos2(center.x + EYEDROP_OUTER_RADIUS, center.y),
        ],
        stroke,
    );
    painter.line_segment(
        [
            egui::pos2(center.x, center.y - EYEDROP_OUTER_RADIUS),
            egui::pos2(center.x, center.y + EYEDROP_OUTER_RADIUS),
        ],
        stroke,
    );
}
