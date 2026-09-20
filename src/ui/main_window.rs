use eframe::egui;

use crate::clipboard;
use crate::color::ColorFormat;
use crate::color::okhsl::Okhsl;
use crate::color::parse;
use crate::config::Config;
use crate::ui::picker::{Event, PickerController};
use crate::ui::slider;
use crate::ui::theme::{self, color32, contrast_color32};

const SLIDER_WIDTH: f32 = 26.0;
const SLIDER_MIN_HEIGHT: f32 = 80.0;
const LABEL_ROW_HEIGHT: f32 = 20.0;
const SLIDER_PANEL_MARGIN: f32 = 8.0;
const SLIDER_PANEL_ID: &str = "colorpickle-sliders";
const HUE_MAX_DEGREES: f32 = 360.0;
const HUE_FRACTION_MAX: f32 = 1.0 - f32::EPSILON;
const DEFAULT_HUE_DEGREES: f32 = 180.0;
const DEFAULT_SATURATION: f32 = 0.5;
const DEFAULT_LIGHTNESS: f32 = 0.5;
const EYEDROP_ICON_SIZE: f32 = 22.0;
const EYEDROP_OUTER_RADIUS: f32 = 8.0;
const EYEDROP_INNER_RADIUS: f32 = 2.5;
const EYEDROP_STROKE_WIDTH: f32 = 1.5;
const COPY_ICON_SIZE: f32 = 18.0;
const COPY_ICON_STROKE_WIDTH: f32 = 1.5;
const COPY_ICON_OFFSET: f32 = 3.0;
const COPY_ICON_CORNER_RADIUS: u8 = 2;
const SATURATION_TOOLTIP: &str = "Saturation is perceptual (Okhsl-normalised), so its visual effect varies slightly with lightness.";
const FORMAT_KEYS: [egui::Key; 8] = [
    egui::Key::Num1,
    egui::Key::Num2,
    egui::Key::Num3,
    egui::Key::Num4,
    egui::Key::Num5,
    egui::Key::Num6,
    egui::Key::Num7,
    egui::Key::Num8,
];

pub struct MainWindow {
    config: Config,
    color: Okhsl,
    input: String,
    input_editing: bool,
    status: Option<String>,
    picker: PickerController,
}

impl MainWindow {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            color: Okhsl::new(DEFAULT_HUE_DEGREES, DEFAULT_SATURATION, DEFAULT_LIGHTNESS),
            input: String::new(),
            input_editing: false,
            status: None,
            picker: PickerController::new(),
        }
    }

    fn apply_input(&mut self) {
        match parse::parse(&self.input) {
            Some(color) => {
                self.color = color;
                self.status = Some("parsed color".to_owned());
            }
            None => self.status = Some("unrecognized color".to_owned()),
        }
        self.input = self.config.default_format.format(self.color);
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

        let background = color32(self.color);
        let foreground = contrast_color32(self.color);

        let mut hue = self.color.hue() / HUE_MAX_DEGREES;
        let mut saturation = self.color.saturation();
        let mut lightness = self.color.lightness();

        let panel_width =
            3.0 * SLIDER_WIDTH + 2.0 * ui.spacing().item_spacing.x + 2.0 * SLIDER_PANEL_MARGIN;
        egui::Panel::right(SLIDER_PANEL_ID)
            .resizable(false)
            .exact_size(panel_width)
            .frame(
                egui::Frame::NONE
                    .fill(background)
                    .inner_margin(SLIDER_PANEL_MARGIN),
            )
            .show(ui, |ui| {
                let slider_height =
                    (ui.available_height() - LABEL_ROW_HEIGHT).max(SLIDER_MIN_HEIGHT);
                let slider_size = egui::vec2(SLIDER_WIDTH, slider_height);
                ui.horizontal_top(|ui| {
                    let hue_saturation = saturation;
                    let hue_lightness = lightness;
                    slider_column(ui, "H", foreground, &mut hue, slider_size, move |value| {
                        color32(Okhsl::new(
                            value * HUE_MAX_DEGREES,
                            hue_saturation,
                            hue_lightness,
                        ))
                    });

                    let saturation_hue = hue * HUE_MAX_DEGREES;
                    let saturation_lightness = lightness;
                    slider_column(
                        ui,
                        "S",
                        foreground,
                        &mut saturation,
                        slider_size,
                        move |value| {
                            color32(Okhsl::new(saturation_hue, value, saturation_lightness))
                        },
                    )
                    .on_hover_text(SATURATION_TOOLTIP);

                    let lightness_hue = hue * HUE_MAX_DEGREES;
                    let lightness_saturation = saturation;
                    slider_column(
                        ui,
                        "L",
                        foreground,
                        &mut lightness,
                        slider_size,
                        move |value| {
                            color32(Okhsl::new(lightness_hue, lightness_saturation, value))
                        },
                    );
                });
            });

        self.color = Okhsl::new(
            hue.min(HUE_FRACTION_MAX) * HUE_MAX_DEGREES,
            saturation,
            lightness,
        );

        let panel_frame = egui::Frame::central_panel(ui.style()).fill(background);
        egui::CentralPanel::default()
            .frame(panel_frame)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    open_picker |= draw_picker_launcher(ui, self.color);
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.input)
                            .desired_width(ui.available_width())
                            .hint_text("color"),
                    );
                    if response.gained_focus() {
                        self.input_editing = true;
                    }
                    if response.lost_focus() {
                        self.input_editing = false;
                        self.apply_input();
                    }
                    if !self.input_editing {
                        self.input = self.config.default_format.format(self.color);
                    }
                });

                ui.horizontal_wrapped(|ui| {
                    draw_copy_icon(ui, foreground);
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

        if !ctx.egui_wants_keyboard_input() && !self.picker.is_busy() {
            for (format, key) in ColorFormat::ALL.iter().zip(FORMAT_KEYS) {
                if ctx.input(|input| input.key_pressed(key)) {
                    self.copy(*format);
                }
            }
        }

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

fn slider_column(
    ui: &mut egui::Ui,
    label: &str,
    label_color: egui::Color32,
    value: &mut f32,
    size: egui::Vec2,
    gradient: impl Fn(f32) -> egui::Color32,
) -> egui::Response {
    ui.vertical(|ui| {
        ui.colored_label(label_color, label);
        slider::vertical(ui, size, value, gradient)
    })
    .inner
}

fn draw_copy_icon(ui: &mut egui::Ui, color: egui::Color32) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(COPY_ICON_SIZE, COPY_ICON_SIZE),
        egui::Sense::hover(),
    );
    let painter = ui.painter();
    let stroke = egui::Stroke::new(COPY_ICON_STROKE_WIDTH, color);
    let corner = egui::CornerRadius::same(COPY_ICON_CORNER_RADIUS);
    let size = egui::vec2(
        rect.width() - COPY_ICON_OFFSET,
        rect.height() - COPY_ICON_OFFSET,
    );
    let front = egui::Rect::from_min_size(rect.min, size);
    let back = front.translate(egui::vec2(COPY_ICON_OFFSET, COPY_ICON_OFFSET));
    painter.rect_stroke(back, corner, stroke, egui::StrokeKind::Inside);
    painter.rect_stroke(front, corner, stroke, egui::StrokeKind::Inside);
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
