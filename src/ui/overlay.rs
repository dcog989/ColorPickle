use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use anyhow::Result;
use eframe::egui;
use palette::Srgb;

use crate::capture;
use crate::clipboard;
use crate::color::okhsl::Okhsl;
use crate::config::Config;
use crate::ui::pixels::{average_rgb8, pixel_at, pixel_bounds};

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
    rect: capture::DesktopRect,
    texture: Option<egui::TextureHandle>,
    uses_portal_fallback: bool,
    drag_anchor: Option<egui::Pos2>,
}

impl Session {
    pub fn new(captured: capture::Capture) -> Self {
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
            rect: captured.rect,
            texture: None,
            uses_portal_fallback: captured.source.uses_portal_fallback(),
            drag_anchor: None,
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
    let session = Session::new(capture::capture()?);
    let outcome: Rc<Cell<Option<PickOutcome>>> = Rc::new(Cell::new(None));

    let options = eframe::NativeOptions {
        viewport: viewport_builder(session.rect),
        persist_window: false,
        ..Default::default()
    };

    let shared = Rc::clone(&outcome);
    eframe::run_native(
        PICKER_TITLE,
        options,
        Box::new(move |_cc| {
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
    session: &mut Session,
    viewport: egui::ViewportId,
) -> Option<PickOutcome> {
    ctx.show_viewport_immediate(viewport, viewport_builder(session.rect), |ui, _class| {
        draw(ui.ctx(), session)
    })
}

fn viewport_builder(rect: capture::DesktopRect) -> egui::ViewportBuilder {
    egui::ViewportBuilder::default()
        .with_title(PICKER_TITLE)
        .with_app_id(PICKER_VIEWPORT)
        .with_decorations(false)
        .with_always_on_top()
        .with_clamp_size_to_monitor_size(false)
        .with_position(egui::pos2(rect.x as f32, rect.y as f32))
        .with_inner_size(egui::vec2(rect.width as f32, rect.height as f32))
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

    if primary_pressed
        && let Some(position) = pointer_pos
    {
        session.drag_anchor = Some(position);
    }

    let pointer = pointer_pos?;

    let region = session.drag_anchor.and_then(|anchor| {
        let moved = (pointer - anchor).length() >= DRAG_THRESHOLD;
        moved.then(|| egui::Rect::from_two_pos(anchor, pointer))
    });
    let dragging = region.is_some();

    let magnifier_center = magnifier_center(screen, pointer, dragging);
    draw_magnifier(
        &painter,
        texture_id,
        magnifier_center,
        uv_at(screen, pointer),
        egui::vec2(
            session.image.size[0] as f32,
            session.image.size[1] as f32,
        ),
    );
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
        return Some(PickOutcome::Picked(color));
    }
    if secondary_clicked || escape_pressed {
        session.drag_anchor = None;
        return Some(PickOutcome::Dismissed);
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
    let marker = egui::Rect::from_center_size(
        marker_center,
        egui::vec2(MAGNIFIER_ZOOM, MAGNIFIER_ZOOM),
    );
    painter.rect_stroke(
        marker,
        egui::CornerRadius::ZERO,
        egui::Stroke::new(CROSSHAIR_WIDTH, SHADOW_COLOR),
        egui::StrokeKind::Outside,
    );
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
