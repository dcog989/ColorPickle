use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use anyhow::Result;
use eframe::egui;
use palette::Srgb;

use crate::capture;
use crate::clipboard;
use crate::color::ColorFormat;
use crate::color::okhsl::Okhsl;
use crate::config::Config;
use crate::ui::pixels::{average_rgb8, pixel_at, pixel_bounds};
use crate::ui::theme;

const PICKER_TITLE: &str = "ColorPickle";
const PICKER_VIEWPORT: &str = "colorpickle-picker";
const MAGNIFIER_SIZE: f32 = 180.0;
const MAGNIFIER_ZOOM: f32 = 4.0;
const MAGNIFIER_SOURCE_PIXELS: f32 = MAGNIFIER_SIZE / MAGNIFIER_ZOOM;
const DRAG_MAGNIFIER_MARGIN: f32 = 24.0;
const CROSSHAIR_ARM: f32 = 8.0;
const CROSSHAIR_WIDTH: f32 = 1.0;
const DRAG_THRESHOLD: f32 = 4.0;
const BANNER_FONT_SIZE: f32 = 14.0;
const BANNER_TOP_MARGIN: f32 = 16.0;
const VALUE_FONT_SIZE: f32 = 15.0;
const VALUE_GAP: f32 = 10.0;
const VALUE_MARGIN: f32 = 8.0;
const VALUE_PADDING: f32 = 8.0;
const VALUE_SWATCH_SIZE: f32 = 16.0;
const VALUE_CORNER_RADIUS: u8 = 6;
const VALUE_SWATCH_RADIUS: u8 = 2;
const VALUE_CHIP_FILL: egui::Color32 = egui::Color32::from_rgba_unmultiplied_const(0, 0, 0, 200);
const VALUE_TEXT_COLOR: egui::Color32 = egui::Color32::WHITE;
const STROKE_COLOR: egui::Color32 = egui::Color32::WHITE;
const SHADOW_COLOR: egui::Color32 = egui::Color32::BLACK;
const SELECTION_FILL: egui::Color32 =
    egui::Color32::from_rgba_unmultiplied_const(255, 255, 255, 40);
const RGB_MAX: f32 = 255.0;
const FRAME_TEXTURE: &str = "picker-frame";
const FULL_UV: egui::Rect = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));

#[derive(Debug, Clone, Copy)]
pub enum PickOutcome {
    Picked(Okhsl),
    Dismissed,
}

pub struct Session {
    image: Arc<egui::ColorImage>,
    texture: Option<egui::TextureHandle>,
    uses_portal_fallback: bool,
    output: capture::OutputInfo,
    drag_anchor: Option<egui::Pos2>,
    format: ColorFormat,
}

impl Session {
    pub fn new(captured: capture::Capture, format: ColorFormat) -> Self {
        let size = [
            captured.image.width() as usize,
            captured.image.height() as usize,
        ];
        let image = Arc::new(egui::ColorImage::from_rgba_unmultiplied(
            size,
            captured.image.as_raw(),
        ));
        Self {
            image,
            texture: None,
            uses_portal_fallback: captured.source.uses_portal_fallback(),
            output: captured.output,
            drag_anchor: None,
            format,
        }
    }

    fn texture_id(&mut self, ctx: &egui::Context) -> egui::TextureId {
        if let Some(texture) = &self.texture {
            return texture.id();
        }
        let max_side = ctx.input(|input| input.max_texture_side).max(1);
        if self.image.size[0] > max_side || self.image.size[1] > max_side {
            tracing::warn!(
                width = self.image.size[0],
                height = self.image.size[1],
                max_side,
                "overlay: frame exceeds the maximum texture size; downscaling"
            );
            self.image = Arc::new(downscale_to_fit(&self.image, max_side));
        }
        tracing::info!(
            width = self.image.size[0],
            height = self.image.size[1],
            "overlay: uploading frame texture"
        );
        let texture = ctx.load_texture(
            FRAME_TEXTURE,
            Arc::clone(&self.image),
            egui::TextureOptions::NEAREST,
        );
        let id = texture.id();
        self.texture = Some(texture);
        id
    }
}

fn downscale_to_fit(image: &egui::ColorImage, max_side: usize) -> egui::ColorImage {
    let [width, height] = image.size;
    let scale = max_side as f32 / width.max(height) as f32;
    let new_width = ((width as f32 * scale).floor() as usize).clamp(1, max_side);
    let new_height = ((height as f32 * scale).floor() as usize).clamp(1, max_side);

    let mut pixels = Vec::with_capacity(new_width * new_height);
    for y in 0..new_height {
        let source_y = y * height / new_height;
        for x in 0..new_width {
            let source_x = x * width / new_width;
            pixels.push(image.pixels[source_y * width + source_x]);
        }
    }
    egui::ColorImage::new([new_width, new_height], pixels)
}

pub fn run(config: Config) -> Result<Option<PickOutcome>> {
    let captured = capture::capture()?;
    let output = captured.output.clone();
    let session = Session::new(captured, config.default_format);
    let outcome: Rc<Cell<Option<PickOutcome>>> = Rc::new(Cell::new(None));

    let options = eframe::NativeOptions {
        viewport: viewport_builder(None),
        persist_window: false,
        ..Default::default()
    };

    let shared = Rc::clone(&outcome);
    eframe::run_native(
        PICKER_TITLE,
        options,
        Box::new(move |cc| {
            let monitor = cc
                .winit_window()
                .and_then(|window| resolve_monitor(window, &output));
            cc.egui_ctx
                .send_viewport_cmd(egui::ViewportCommand::SetMonitor(monitor.unwrap_or(0)));
            Ok(Box::new(StandalonePicker {
                session,
                config,
                outcome: shared,
            }))
        }),
    )
    .map_err(|error| anyhow::anyhow!("failed to run the picker: {error}"))?;

    Ok(outcome.get())
}

pub fn show(
    ctx: &egui::Context,
    frame: &eframe::Frame,
    session: &mut Session,
    viewport: egui::ViewportId,
) -> Option<PickOutcome> {
    let output = session.output.clone();
    let monitor = frame
        .winit_window()
        .and_then(|window| resolve_monitor(window, &output));
    ctx.show_viewport_immediate(viewport, viewport_builder(monitor), |ui, _class| {
        draw(ui.ctx(), session)
    })
}

fn viewport_builder(monitor: Option<usize>) -> egui::ViewportBuilder {
    let builder = egui::ViewportBuilder::default()
        .with_title(PICKER_TITLE)
        .with_app_id(PICKER_VIEWPORT)
        .with_fullscreen(true)
        .with_decorations(false)
        .with_always_on_top();
    match monitor {
        Some(index) => builder.with_monitor(index),
        None => builder,
    }
}

fn resolve_monitor(window: &winit::window::Window, output: &capture::OutputInfo) -> Option<usize> {
    let monitors: Vec<winit::monitor::MonitorHandle> = window.available_monitors().collect();
    if let Some(name) = &output.name
        && let Some(index) = monitors
            .iter()
            .position(|monitor| monitor.name().as_ref() == Some(name))
    {
        return Some(index);
    }
    if let Some(position) = output.position {
        let (x, y) = position;
        return monitors
            .iter()
            .position(|monitor| monitor.position().x == x && monitor.position().y == y);
    }
    None
}

struct StandalonePicker {
    session: Session,
    config: Config,
    outcome: Rc<Cell<Option<PickOutcome>>>,
}

impl eframe::App for StandalonePicker {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let Some(outcome) = draw(ui.ctx(), &mut self.session) else {
            return;
        };
        if let PickOutcome::Picked(color) = outcome
            && let Err(error) = clipboard::copy_color(self.config.default_format, color)
        {
            tracing::warn!(?error, "clipboard write failed");
        }
        self.outcome.set(Some(outcome));
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
    }
}

fn draw(ctx: &egui::Context, session: &mut Session) -> Option<PickOutcome> {
    let texture_id = session.texture_id(ctx);
    let screen = ctx.input(|input| input.viewport_rect());
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new(PICKER_VIEWPORT),
    ));
    painter.image(texture_id, screen, FULL_UV, egui::Color32::WHITE);

    let (
        pointer_pos,
        primary_pressed,
        primary_released,
        primary_down,
        secondary_clicked,
        escape_pressed,
    ) = ctx.input(|input| {
        (
            input.pointer.hover_pos(),
            input.pointer.primary_pressed(),
            input.pointer.primary_released(),
            input.pointer.primary_down(),
            input.pointer.secondary_clicked(),
            input.key_pressed(egui::Key::Escape),
        )
    });

    if secondary_clicked || escape_pressed {
        session.drag_anchor = None;
        return Some(PickOutcome::Dismissed);
    }

    if primary_pressed && let Some(position) = pointer_pos {
        session.drag_anchor = Some(position);
    }

    let pointer = pointer_pos?;

    let region = session.drag_anchor.and_then(|anchor| {
        let moved = (pointer - anchor).length() >= DRAG_THRESHOLD;
        moved.then(|| egui::Rect::from_two_pos(anchor, pointer))
    });
    let dragging = region.is_some();

    let color = match region {
        Some(rect) => {
            let bounds = pixel_bounds(
                &session.image,
                uv_at(screen, rect.min),
                uv_at(screen, rect.max),
            );
            average_color(&session.image, bounds)
        }
        None => {
            let (pixel_x, pixel_y) = pixel_at(&session.image, uv_at(screen, pointer));
            average_color(&session.image, (pixel_x, pixel_y, pixel_x, pixel_y))
        }
    };

    let magnifier_center = magnifier_center(screen, pointer, dragging);
    draw_magnifier(
        &painter,
        texture_id,
        magnifier_center,
        uv_at(screen, pointer),
        egui::vec2(session.image.size[0] as f32, session.image.size[1] as f32),
    );
    draw_value(&painter, screen, magnifier_center, color, session.format);
    if let Some(rect) = region {
        draw_selection(&painter, rect);
    } else {
        draw_crosshair(&painter, pointer);
    }
    if session.uses_portal_fallback {
        draw_banner(&painter, screen);
    }

    if primary_released {
        session.drag_anchor = None;
        return Some(PickOutcome::Picked(color));
    }
    if !primary_down {
        session.drag_anchor = None;
    }
    None
}

fn uv_at(screen: egui::Rect, point: egui::Pos2) -> egui::Pos2 {
    egui::pos2(
        ((point.x - screen.min.x) / screen.width()).clamp(0.0, 1.0),
        ((point.y - screen.min.y) / screen.height()).clamp(0.0, 1.0),
    )
}

fn magnifier_center(screen: egui::Rect, pointer: egui::Pos2, dragging: bool) -> egui::Pos2 {
    if dragging {
        egui::pos2(
            screen.right() - MAGNIFIER_SIZE / 2.0 - DRAG_MAGNIFIER_MARGIN,
            screen.bottom() - MAGNIFIER_SIZE / 2.0 - DRAG_MAGNIFIER_MARGIN,
        )
    } else {
        pointer
    }
}

fn draw_magnifier(
    painter: &egui::Painter,
    texture_id: egui::TextureId,
    center: egui::Pos2,
    uv: egui::Pos2,
    source_size: egui::Vec2,
) {
    let half_x = (MAGNIFIER_SOURCE_PIXELS * 0.5 / source_size.x).min(0.5);
    let half_y = (MAGNIFIER_SOURCE_PIXELS * 0.5 / source_size.y).min(0.5);
    let source_center = egui::pos2(
        uv.x.clamp(half_x, 1.0 - half_x),
        uv.y.clamp(half_y, 1.0 - half_y),
    );
    let source = egui::Rect::from_min_max(
        egui::pos2(source_center.x - half_x, source_center.y - half_y),
        egui::pos2(source_center.x + half_x, source_center.y + half_y),
    );
    let target = egui::Rect::from_center_size(center, egui::vec2(MAGNIFIER_SIZE, MAGNIFIER_SIZE));

    painter.image(texture_id, target, source, egui::Color32::WHITE);
    painter.rect_stroke(
        target,
        egui::CornerRadius::ZERO,
        egui::Stroke::new(CROSSHAIR_WIDTH * 2.0, STROKE_COLOR),
        egui::StrokeKind::Inside,
    );

    let marker_fraction = egui::vec2(
        (uv.x - source.min.x) / source.width(),
        (uv.y - source.min.y) / source.height(),
    );
    let marker_center = egui::pos2(
        target.min.x + marker_fraction.x * target.width(),
        target.min.y + marker_fraction.y * target.height(),
    );
    let marker =
        egui::Rect::from_center_size(marker_center, egui::vec2(MAGNIFIER_ZOOM, MAGNIFIER_ZOOM));
    painter.rect_stroke(
        marker,
        egui::CornerRadius::ZERO,
        egui::Stroke::new(CROSSHAIR_WIDTH, SHADOW_COLOR),
        egui::StrokeKind::Outside,
    );
}

fn draw_value(
    painter: &egui::Painter,
    screen: egui::Rect,
    magnifier_center: egui::Pos2,
    color: Okhsl,
    format: ColorFormat,
) {
    let galley = painter.layout_no_wrap(
        format.format(color),
        egui::FontId::proportional(VALUE_FONT_SIZE),
        VALUE_TEXT_COLOR,
    );
    let text_size = galley.size();
    let chip_size = egui::vec2(
        text_size.x + VALUE_SWATCH_SIZE + VALUE_PADDING * 3.0,
        text_size.y.max(VALUE_SWATCH_SIZE) + VALUE_PADDING * 2.0,
    );

    let half = MAGNIFIER_SIZE / 2.0 + VALUE_GAP;
    let below = magnifier_center.y + half + chip_size.y / 2.0;
    let above = magnifier_center.y - half - chip_size.y / 2.0;
    let highest = screen.top() + VALUE_MARGIN + chip_size.y / 2.0;
    let lowest = screen.bottom() - VALUE_MARGIN - chip_size.y / 2.0;
    let y = if below <= lowest {
        below
    } else {
        above.max(highest)
    };
    let x = magnifier_center.x.clamp(
        screen.left() + VALUE_MARGIN + chip_size.x / 2.0,
        screen.right() - VALUE_MARGIN - chip_size.x / 2.0,
    );
    let chip = egui::Rect::from_center_size(egui::pos2(x, y), chip_size);

    painter.rect_filled(
        chip,
        egui::CornerRadius::same(VALUE_CORNER_RADIUS),
        VALUE_CHIP_FILL,
    );

    let swatch = egui::Rect::from_center_size(
        egui::pos2(
            chip.left() + VALUE_PADDING + VALUE_SWATCH_SIZE / 2.0,
            chip.center().y,
        ),
        egui::vec2(VALUE_SWATCH_SIZE, VALUE_SWATCH_SIZE),
    );
    painter.rect_filled(
        swatch,
        egui::CornerRadius::same(VALUE_SWATCH_RADIUS),
        theme::color32(color),
    );
    painter.rect_stroke(
        swatch,
        egui::CornerRadius::same(VALUE_SWATCH_RADIUS),
        egui::Stroke::new(CROSSHAIR_WIDTH, STROKE_COLOR),
        egui::StrokeKind::Inside,
    );

    let text_pos = egui::pos2(
        swatch.right() + VALUE_PADDING,
        chip.center().y - text_size.y / 2.0,
    );
    painter.galley(text_pos, galley, VALUE_TEXT_COLOR);
}

fn draw_selection(painter: &egui::Painter, rect: egui::Rect) {
    painter.rect_filled(rect, egui::CornerRadius::ZERO, SELECTION_FILL);
    painter.rect_stroke(
        rect,
        egui::CornerRadius::ZERO,
        egui::Stroke::new(CROSSHAIR_WIDTH * 2.0, STROKE_COLOR),
        egui::StrokeKind::Inside,
    );
}

fn draw_crosshair(painter: &egui::Painter, pointer: egui::Pos2) {
    let stroke = egui::Stroke::new(CROSSHAIR_WIDTH, STROKE_COLOR);
    painter.line_segment(
        [
            egui::pos2(pointer.x - CROSSHAIR_ARM, pointer.y),
            egui::pos2(pointer.x + CROSSHAIR_ARM, pointer.y),
        ],
        stroke,
    );
    painter.line_segment(
        [
            egui::pos2(pointer.x, pointer.y - CROSSHAIR_ARM),
            egui::pos2(pointer.x, pointer.y + CROSSHAIR_ARM),
        ],
        stroke,
    );
}

fn draw_banner(painter: &egui::Painter, screen: egui::Rect) {
    painter.text(
        egui::pos2(screen.center().x, screen.min.y + BANNER_TOP_MARGIN),
        egui::Align2::CENTER_TOP,
        "picking from a static screenshot",
        egui::FontId::proportional(BANNER_FONT_SIZE),
        STROKE_COLOR,
    );
}

fn average_color(image: &egui::ColorImage, bounds: (u32, u32, u32, u32)) -> Okhsl {
    let [red, green, blue] = average_rgb8(image, bounds);
    Okhsl::from_srgb(Srgb::new(
        f32::from(red) / RGB_MAX,
        f32::from(green) / RGB_MAX,
        f32::from(blue) / RGB_MAX,
    ))
}
