use eframe::egui;

use crate::clipboard;
use crate::color::ColorFormat;
use crate::color::harmony::Harmony;
use crate::color::okhsl::Okhsl;
use crate::color::parse;
use crate::config::Config;
use crate::ui::picker::{Event, PickerController};
use crate::ui::theme::{self, color32, contrast_color32};
use crate::ui::widgets;

mod history;
mod keys;
mod settings_panel;
mod slider_panel;
mod toast;

use self::history::History;
use self::keys::{ColorKey, InputKey, ThemeKey};
use self::toast::Toast;

const PANEL_MARGIN: f32 = 16.0;
const ROW_SPACING: f32 = 18.0;
const ITEM_SPACING: f32 = 10.0;
const INPUT_FONT_SIZE: f32 = 26.0;
const INPUT_MARGIN_X: i8 = 12;
const INPUT_MARGIN_Y: i8 = 10;
const MIN_WINDOW_HEIGHT: f32 = 420.0;
const DEFAULT_HUE_DEGREES: f32 = 180.0;
const DEFAULT_SATURATION: f32 = 0.5;
const DEFAULT_LIGHTNESS: f32 = 0.5;
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

const _: () = assert!(FORMAT_KEYS.len() == ColorFormat::ALL.len());

pub struct MainWindow {
    config: Config,
    color: Okhsl,
    input: String,
    input_editing: bool,
    input_dirty: bool,
    history: History,
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
            history: History::default(),
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
        self.history.push(color);
        let value = self.config.format(color);
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
        self.toast = Some(Toast::new(message, self.now));
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
        self.input = self.config.format(self.color);
        self.applied_input = Some(InputKey {
            format: self.config.default_format,
            color: ColorKey::new(self.color),
        });
    }

    fn copy(&mut self, format: ColorFormat) {
        match clipboard::copy_color(format, self.color) {
            Ok(value) => self.set_toast(format!("Copied {value}")),
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
                self.history.push(color);
                match clipboard::copy_color(self.config.default_format, color) {
                    Ok(value) => self.set_toast(format!("Picked {value}")),
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

        let panel_width = slider_panel::show(ui, &mut self.color, background, foreground);
        if settings_panel::show(ui, &mut self.config, background, foreground) {
            self.persist_config();
        }
        let open_picker = self.show_central(ui, background, foreground, panel_width);

        self.handle_shortcuts(&ctx);

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
    fn show_central(
        &mut self,
        ui: &mut egui::Ui,
        background: egui::Color32,
        foreground: egui::Color32,
        panel_width: f32,
    ) -> bool {
        let ctx = ui.ctx().clone();
        let mut open_picker = false;

        let input_font = egui::FontId::proportional(INPUT_FONT_SIZE);
        let input_height =
            ctx.fonts_mut(|fonts| fonts.row_height(&input_font)) + 2.0 * f32::from(INPUT_MARGIN_Y);

        let panel_frame = egui::Frame::central_panel(ui.style())
            .inner_margin(egui::Margin {
                left: PANEL_MARGIN as i8,
                right: PANEL_MARGIN as i8,
                top: PANEL_MARGIN as i8,
                bottom: settings_panel::MARGIN_Y,
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
                            self.input = self.config.format(self.color);
                            self.applied_input = Some(input_key);
                        }
                    }
                });

                ui.add_space(ROW_SPACING);

                let color = self.color;
                let format_row = ui.horizontal(|ui| {
                    widgets::copy_icon(ui, foreground);
                    for format in ColorFormat::ALL {
                        let response = ui.button(format.label()).on_hover_ui(|ui| {
                            ui.label(format!("Copy {}", format.format(color)));
                        });
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
                    for &color in self.history.colors() {
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

        open_picker
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let keyboard_captured = ctx.egui_wants_keyboard_input();
        if keyboard_captured || self.picker.is_busy() {
            return;
        }

        for (format, key) in ColorFormat::ALL.iter().zip(FORMAT_KEYS) {
            if ctx.input(|input| input.key_pressed(key)) {
                self.copy(*format);
            }
        }

        if ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn show_toast(&mut self, ctx: &egui::Context) {
        if self
            .toast
            .as_ref()
            .is_some_and(|toast| toast.is_expired(self.now))
        {
            self.toast = None;
            return;
        }

        let Some(toast) = self.toast.as_ref() else {
            return;
        };
        ctx.request_repaint_after(toast.remaining(self.now));
        toast.show(ctx);
    }
}
