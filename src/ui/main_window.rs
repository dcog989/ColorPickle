use std::time::Duration;

use eframe::egui;

use crate::cli::LaunchMode;
use crate::clipboard;
use crate::color::ColorFormat;
use crate::color::harmony::Harmony;
use crate::color::okhsl::Okhsl;
use crate::color::parse;
use crate::config::{Config, Theme};
use crate::ui::picker::{Event, PickerController};
use crate::ui::slider;
use crate::ui::theme::{self, color32, contrast_color32};
use crate::ui::widgets;

const SLIDER_WIDTH: f32 = 30.0;
const SLIDER_PANEL_MARGIN: f32 = 12.0;
const SLIDER_PANEL_ID: &str = "colorpickle-sliders";
const SETTINGS_PANEL_ID: &str = "colorpickle-settings";
const SETTINGS_PANEL_MARGIN_X: i8 = 16;
const SETTINGS_PANEL_MARGIN_Y: i8 = 10;
const PANEL_MARGIN: f32 = 16.0;
const ROW_SPACING: f32 = 18.0;
const ITEM_SPACING: f32 = 10.0;
const INPUT_FONT_SIZE: f32 = 26.0;
const INPUT_MARGIN_X: i8 = 12;
const INPUT_MARGIN_Y: i8 = 10;
const MIN_WINDOW_HEIGHT: f32 = 420.0;
const HUE_MAX_DEGREES: f32 = 360.0;
const HUE_FRACTION_MAX: f32 = 1.0 - f32::EPSILON;
const DEFAULT_HUE_DEGREES: f32 = 180.0;
const DEFAULT_SATURATION: f32 = 0.5;
const DEFAULT_LIGHTNESS: f32 = 0.5;
const HISTORY_LIMIT: usize = 8;
const TOAST_DURATION: f64 = 2.5;
const TOAST_BOTTOM_MARGIN: f32 = 84.0;
const TOAST_MARGIN_X: i8 = 16;
const TOAST_MARGIN_Y: i8 = 10;
const TOAST_ID: &str = "colorpickle-toast";
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

struct Toast {
    message: String,
    expires_at: f64,
}

#[derive(Clone, Copy, PartialEq)]
struct ColorKey {
    hue: f32,
    saturation: f32,
    lightness: f32,
}

impl ColorKey {
    fn new(color: Okhsl) -> Self {
        Self {
            hue: color.hue(),
            saturation: color.saturation(),
            lightness: color.lightness(),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct ThemeKey {
    theme: Theme,
    system: Option<egui::Theme>,
    color: ColorKey,
}

#[derive(Clone, Copy, PartialEq)]
struct InputKey {
    format: ColorFormat,
    color: ColorKey,
}

pub struct MainWindow {
    config: Config,
    color: Okhsl,
    input: String,
    input_editing: bool,
    input_dirty: bool,
    history: Vec<Okhsl>,
    harmony: Harmony,
    toast: Option<Toast>,
    now: f64,
    pending_toast: Option<String>,
    picker: PickerController,
    applied_theme: Option<ThemeKey>,
    applied_input: Option<InputKey>,
    min_inner_size: Option<egui::Vec2>,
}

impl MainWindow {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            color: Okhsl::new(DEFAULT_HUE_DEGREES, DEFAULT_SATURATION, DEFAULT_LIGHTNESS),
            input: String::new(),
            input_editing: false,
            input_dirty: false,
            history: Vec::new(),
            harmony: Harmony::default(),
            toast: None,
            now: 0.0,
            pending_toast: None,
            picker: PickerController::new(),
            applied_theme: None,
            applied_input: None,
            min_inner_size: None,
        }
    }

    pub fn with_initial(mut self, color: Okhsl) -> Self {
        self.color = color;
        self.push_history(color);
        let value = self.config.default_format.format(color);
        self.pending_toast = Some(format!("Picked {value}"));
        self
    }

    pub fn apply_theme(&mut self, ctx: &egui::Context) {
        theme::apply(ctx, self.config.theme, self.color);
        self.applied_theme = Some(self.theme_key(ctx));
    }

    fn theme_key(&self, ctx: &egui::Context) -> ThemeKey {
        ThemeKey {
            theme: self.config.theme,
            system: ctx.system_theme(),
            color: ColorKey::new(self.color),
        }
    }

    fn set_toast(&mut self, message: impl Into<String>) {
        self.toast = Some(Toast {
            message: message.into(),
            expires_at: self.now + TOAST_DURATION,
        });
    }

    fn push_history(&mut self, color: Okhsl) {
        self.history
            .retain(|existing| existing.to_srgb8() != color.to_srgb8());
        self.history.insert(0, color);
        self.history.truncate(HISTORY_LIMIT);
    }

    fn persist_config(&mut self) {
        if let Err(error) = self.config.save() {
            self.set_toast(format!("Could not save settings: {error}"));
        }
    }

    fn apply_input(&mut self) {
        match parse::parse(&self.input) {
            Some(color) => self.color = color,
            None => self.set_toast("Unrecognized color"),
        }
        self.input = self.config.default_format.format(self.color);
        self.applied_input = Some(InputKey {
            format: self.config.default_format,
            color: ColorKey::new(self.color),
        });
    }

    fn copy(&mut self, format: ColorFormat) {
        let value = format.format(self.color);
        match clipboard::set_text(value.clone()) {
            Ok(()) => self.set_toast(format!("Copied {value}")),
            Err(error) => self.set_toast(format!("Copy failed: {error}")),
        }
    }

    fn open_picker(&mut self, ctx: &egui::Context) {
        if self.picker.request(ctx) {
            self.set_toast("Capturing screen...");
        }
    }

    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Ready => self.set_toast("Picking from screen..."),
            Event::CaptureFailed(error) => self.set_toast(format!("Capture failed: {error}")),
            Event::ThreadStopped => self.set_toast("Capture thread stopped unexpectedly"),
            Event::Picked(color) => {
                self.color = color;
                self.push_history(color);
                let value = self.config.default_format.format(color);
                match clipboard::set_text(value.clone()) {
                    Ok(()) => self.set_toast(format!("Picked {value}")),
                    Err(error) => self.set_toast(format!("Copy failed: {error}")),
                }
            }
            Event::Dismissed => self.set_toast("Picking cancelled"),
        }
    }
}

impl eframe::App for MainWindow {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let mut open_picker = false;

        self.now = ctx.input(|input| input.time);

        if let Some(message) = self.pending_toast.take() {
            self.set_toast(message);
        }

        ui.spacing_mut().item_spacing = egui::vec2(ITEM_SPACING, ITEM_SPACING);
        ui.spacing_mut().button_padding = egui::vec2(ITEM_SPACING, ITEM_SPACING * 0.6);
        let theme_key = self.theme_key(&ctx);
        if self.applied_theme != Some(theme_key) {
            theme::apply(&ctx, self.config.theme, self.color);
            self.applied_theme = Some(theme_key);
        }

        let background = color32(self.color);
        let foreground = contrast_color32(self.color);

        let input_font = egui::FontId::proportional(INPUT_FONT_SIZE);
        let input_height =
            ctx.fonts_mut(|fonts| fonts.row_height(&input_font)) + 2.0 * f32::from(INPUT_MARGIN_Y);

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
                ui.horizontal_top(|ui| {
                    let hue_saturation = saturation;
                    let hue_lightness = lightness;
                    slider::column(ui, "H", foreground, &mut hue, SLIDER_WIDTH, move |value| {
                        color32(Okhsl::new(
                            value * HUE_MAX_DEGREES,
                            hue_saturation,
                            hue_lightness,
                        ))
                    });

                    let saturation_hue = hue * HUE_MAX_DEGREES;
                    let saturation_lightness = lightness;
                    slider::column(
                        ui,
                        "S",
                        foreground,
                        &mut saturation,
                        SLIDER_WIDTH,
                        move |value| {
                            color32(Okhsl::new(saturation_hue, value, saturation_lightness))
                        },
                    )
                    .on_hover_text(SATURATION_TOOLTIP);

                    let lightness_hue = hue * HUE_MAX_DEGREES;
                    let lightness_saturation = saturation;
                    slider::column(
                        ui,
                        "L",
                        foreground,
                        &mut lightness,
                        SLIDER_WIDTH,
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

        egui::Panel::bottom(SETTINGS_PANEL_ID)
            .resizable(false)
            .frame(
                egui::Frame::NONE
                    .fill(background)
                    .inner_margin(egui::Margin::symmetric(
                        SETTINGS_PANEL_MARGIN_X,
                        SETTINGS_PANEL_MARGIN_Y,
                    )),
            )
            .show(ui, |ui| {
                let mut config_changed = false;
                let row_height = widgets::row_height(ui);
                ui.spacing_mut().interact_size.y = row_height;
                ui.horizontal(|ui| {
                    widgets::settings_icon(ui, foreground);
                    egui::ComboBox::from_id_salt("launch-mode")
                        .selected_text(launch_mode_label(self.config.launch_mode))
                        .show_ui(ui, |ui| {
                            for mode in [LaunchMode::UiFirst, LaunchMode::PickerFirst] {
                                if ui
                                    .selectable_value(
                                        &mut self.config.launch_mode,
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
                        .selected_text(self.config.default_format.label())
                        .show_ui(ui, |ui| {
                            for format in ColorFormat::ALL {
                                if ui
                                    .selectable_value(
                                        &mut self.config.default_format,
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
                        .on_hover_text(
                            "Default format: shown in the input field and copied on pick",
                        );
                });
                if config_changed {
                    self.persist_config();
                }
            });

        let panel_frame = egui::Frame::central_panel(ui.style())
            .inner_margin(egui::Margin {
                left: PANEL_MARGIN as i8,
                right: PANEL_MARGIN as i8,
                top: PANEL_MARGIN as i8,
                bottom: SETTINGS_PANEL_MARGIN_Y,
            })
            .fill(background);
        egui::CentralPanel::default()
            .frame(panel_frame)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    open_picker |= widgets::picker_launcher(ui, input_height);
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.input)
                            .font(input_font.clone())
                            .margin(egui::Margin::symmetric(INPUT_MARGIN_X, INPUT_MARGIN_Y))
                            .desired_width(ui.available_width())
                            .hint_text("color"),
                    );
                    if response.gained_focus() {
                        self.input_editing = true;
                    }
                    if response.changed() && self.input_editing {
                        self.input_dirty = true;
                    }
                    if response.lost_focus() {
                        self.input_editing = false;
                        if self.input_dirty {
                            self.input_dirty = false;
                            self.apply_input();
                        }
                    }
                    if !self.input_editing {
                        let input_key = InputKey {
                            format: self.config.default_format,
                            color: ColorKey::new(self.color),
                        };
                        if self.applied_input != Some(input_key) {
                            self.input = self.config.default_format.format(self.color);
                            self.applied_input = Some(input_key);
                        }
                    }
                });

                ui.add_space(ROW_SPACING);

                let color = self.color;
                let format_row = ui.horizontal(|ui| {
                    widgets::copy_icon(ui, foreground);
                    for format in ColorFormat::ALL {
                        let response = ui
                            .button(format.label())
                            .on_hover_ui(|ui| ui.label(format!("Copy {}", format.format(color))));
                        if response.clicked() {
                            self.copy(format);
                        }
                    }
                });

                let required_width =
                    panel_width + 2.0 * PANEL_MARGIN + format_row.response.rect.width();
                let min_inner_size = egui::vec2(required_width, MIN_WINDOW_HEIGHT);
                if self.min_inner_size != Some(min_inner_size) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(min_inner_size));
                    self.min_inner_size = Some(min_inner_size);
                }

                ui.add_space(ROW_SPACING);

                ui.horizontal(|ui| {
                    if widgets::clear_history(ui) {
                        self.history.clear();
                    }
                    let mut selected = None;
                    for &color in &self.history {
                        if widgets::history_swatch(ui, color, foreground) {
                            selected = Some(color);
                        }
                    }
                    if let Some(color) = selected {
                        self.color = color;
                    }
                });

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    ui.spacing_mut().interact_size.y = widgets::row_height(ui);
                    ui.horizontal(|ui| {
                        widgets::palette_icon(ui, foreground);
                        egui::ComboBox::from_id_salt("harmony")
                            .selected_text(self.harmony.label())
                            .show_ui(ui, |ui| {
                                for harmony in Harmony::ALL {
                                    ui.selectable_value(
                                        &mut self.harmony,
                                        harmony,
                                        harmony.label(),
                                    );
                                }
                            })
                            .response
                            .on_hover_text("Colour harmony");
                        for swatch in self.harmony.swatches(self.color) {
                            if widgets::history_swatch(ui, swatch, foreground) {
                                self.color = swatch;
                            }
                        }
                    });
                });
            });

        let keyboard_captured = ctx.egui_wants_keyboard_input();

        if !keyboard_captured && !self.picker.is_busy() {
            for (format, key) in ColorFormat::ALL.iter().zip(FORMAT_KEYS) {
                if ctx.input(|input| input.key_pressed(key)) {
                    self.copy(*format);
                }
            }
        }

        if !keyboard_captured
            && !self.picker.is_busy()
            && ctx.input(|input| input.key_pressed(egui::Key::Escape))
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        if open_picker {
            self.open_picker(&ctx);
        }

        if let Some(event) = self.picker.update(&ctx) {
            self.handle_event(event);
        }

        self.show_toast(&ctx);
    }
}

impl MainWindow {
    fn show_toast(&mut self, ctx: &egui::Context) {
        if self
            .toast
            .as_ref()
            .is_some_and(|toast| self.now >= toast.expires_at)
        {
            self.toast = None;
        }

        let Some(toast) = self.toast.as_ref() else {
            return;
        };
        let remaining = (toast.expires_at - self.now).max(0.0);
        let message = toast.message.clone();
        ctx.request_repaint_after(Duration::from_secs_f64(remaining));

        egui::Area::new(egui::Id::new(TOAST_ID))
            .anchor(
                egui::Align2::CENTER_BOTTOM,
                egui::vec2(0.0, -TOAST_BOTTOM_MARGIN),
            )
            .order(egui::Order::Foreground)
            .interactable(false)
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style())
                    .inner_margin(egui::Margin::symmetric(TOAST_MARGIN_X, TOAST_MARGIN_Y))
                    .show(ui, |ui| {
                        ui.add(egui::Label::new(message).extend());
                    });
            });
    }
}

fn launch_mode_label(mode: LaunchMode) -> &'static str {
    match mode {
        LaunchMode::UiFirst => "UI first",
        LaunchMode::PickerFirst => "Picker first",
    }
}
