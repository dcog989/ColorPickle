use std::sync::{Arc, Mutex};

use anyhow::Result;
use eframe::egui;
use image::RgbaImage;
use palette::Srgb;

use crate::capture;
use crate::clipboard;
use crate::color::okhsl::Okhsl;
use crate::config::Config;

const PICKER_TITLE: &str = "ColorPickle";
const PICKER_VIEWPORT: &str = "colorpickle-picker";
const MAGNIFIER_SIZE: f32 = 180.0;
const MAGNIFIER_ZOOM: f32 = 4.0;
const MAGNIFIER_SOURCE_PIXELS: f32 = MAGNIFIER_SIZE / MAGNIFIER_ZOOM;
const MAGNIFIER_PIXEL_SIZE: f32 = MAGNIFIER_SIZE / MAGNIFIER_SOURCE_PIXELS;
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

pub type CaptureOutcome = capture::CaptureResult<CapturedFrame>;

pub struct CapturedFrame {
    frame: RgbaImage,
    rect: capture::DesktopRect,
    uses_portal_fallback: bool,
}

impl CapturedFrame {
    pub fn capture() -> capture::CaptureResult<Self> {
        Self::capture_with(|| {})
    }

    pub fn capture_with(progress: impl FnOnce()) -> capture::CaptureResult<Self> {
        let backend = capture::detect()?;
        let uses_portal_fallback = backend.uses_portal_fallback();
        if uses_portal_fallback {
            progress();
        }
        let capture = backend.capture_fullscreen()?;
        if capture.image.width() == 0 || capture.image.height() == 0 {
            return Err(capture::CaptureError::EmptyFrame);
        }
        Ok(Self {
            frame: capture.image,
            rect: capture.rect,
            uses_portal_fallback,
        })
    }
}

pub struct Session {
    frame: RgbaImage,
    rect: capture::DesktopRect,
    texture: Option<egui::TextureHandle>,
    uses_portal_fallback: bool,
    drag_anchor: Option<egui::Pos2>,
}

impl Session {
    pub fn new(captured: CapturedFrame) -> Self {
        Self {
            frame: captured.frame,
            rect: captured.rect,
            texture: None,
            uses_portal_fallback: captured.uses_portal_fallback,
            drag_anchor: None,
        }
    }

    fn texture_id(&mut self, ctx: &egui::Context) -> egui::TextureId {
        if let Some(texture) = &self.texture {
            return texture.id();
        }
        let size = [self.frame.width() as usize, self.frame.height() as usize];
        tracing::info!(
            width = size[0],
            height = size[1],
            "overlay: uploading frame texture"
        );
        let image = egui::ColorImage::from_rgba_unmultiplied(size, self.frame.as_raw());
        let texture = ctx.load_texture(FRAME_TEXTURE, image, egui::TextureOptions::NEAREST);
        let id = texture.id();
        self.texture = Some(texture);
        id
    }
}

pub fn run(config: Config) -> Result<Option<PickOutcome>> {
    let session = Session::new(CapturedFrame::capture()?);
    let outcome = Arc::new(Mutex::new(None));

    let options = eframe::NativeOptions {
        viewport: viewport_builder(session.rect),
        persist_window: false,
        ..Default::default()
    };

    let shared = Arc::clone(&outcome);
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

    let outcome = *outcome.lock().expect("picker outcome mutex poisoned");
    Ok(outcome)
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
    outcome: Arc<Mutex<Option<PickOutcome>>>,
}

impl eframe::App for StandalonePicker {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let Some(outcome) = draw(ui.ctx(), &mut self.session) else {
            return;
        };
        if let PickOutcome::Picked(color) = outcome {
            let value = self.config.default_format.format(color);
            if let Err(error) = clipboard::set_text(value) {
                tracing::warn!(?error, "clipboard write failed");
            }
        }
        *self.outcome.lock().expect("picker outcome mutex poisoned") = Some(outcome);
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
    }
}

fn draw(ctx: &egui::Context, session: &mut Session) -> Option<PickOutcome> {
    ctx.request_repaint();

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

    if primary_pressed {
        if let Some(position) = pointer_pos {
            session.drag_anchor = Some(position);
        }
    }

    let Some(pointer) = pointer_pos else {
        return None;
    };

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
        egui::vec2(session.frame.width() as f32, session.frame.height() as f32),
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
                    &session.frame,
                    uv_at(screen, rect.min),
                    uv_at(screen, rect.max),
                );
                average_color(&session.frame, bounds)
            }
            None => {
                let (pixel_x, pixel_y) = pixel_at(&session.frame, uv_at(screen, pointer));
                average_color(&session.frame, (pixel_x, pixel_y, pixel_x, pixel_y))
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

fn pixel_at(frame: &RgbaImage, uv: egui::Pos2) -> (u32, u32) {
    let x = (uv.x * frame.width() as f32).clamp(0.0, frame.width() as f32 - 1.0);
    let y = (uv.y * frame.height() as f32).clamp(0.0, frame.height() as f32 - 1.0);
    (x as u32, y as u32)
}

fn pixel_bounds(frame: &RgbaImage, a: egui::Pos2, b: egui::Pos2) -> (u32, u32, u32, u32) {
    let (x0, y0) = pixel_at(frame, a);
    let (x1, y1) = pixel_at(frame, b);
    (x0.min(x1), y0.min(y1), x0.max(x1), y0.max(y1))
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
        egui::vec2(MAGNIFIER_PIXEL_SIZE, MAGNIFIER_PIXEL_SIZE),
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

fn average_color(frame: &RgbaImage, bounds: (u32, u32, u32, u32)) -> Okhsl {
    let [red, green, blue] = average_rgb8(frame, bounds);
    Okhsl::from_srgb(Srgb::new(
        f32::from(red) / RGB_MAX,
        f32::from(green) / RGB_MAX,
        f32::from(blue) / RGB_MAX,
    ))
}

fn average_rgb8(frame: &RgbaImage, bounds: (u32, u32, u32, u32)) -> [u8; 3] {
    let (start_x, start_y, end_x, end_y) = bounds;
    let mut sums = [0u64; 3];
    let mut count = 0u64;
    for y in start_y..=end_y {
        for x in start_x..=end_x {
            let pixel = frame.get_pixel(x, y);
            sums[0] += u64::from(pixel[0]);
            sums[1] += u64::from(pixel[1]);
            sums[2] += u64::from(pixel[2]);
            count += 1;
        }
    }

    if count == 0 {
        return [0, 0, 0];
    }
    [
        (sums[0] / count) as u8,
        (sums[1] / count) as u8,
        (sums[2] / count) as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::average_rgb8;
    use image::{Rgba, RgbaImage};

    #[test]
    fn averages_a_region_block() {
        let mut frame = RgbaImage::new(4, 4);
        for y in 0..2 {
            for x in 0..2 {
                frame.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }
        assert_eq!(average_rgb8(&frame, (0, 0, 1, 1)), [255, 0, 0]);
    }

    #[test]
    fn averages_partial_coverage() {
        let mut frame = RgbaImage::new(2, 2);
        frame.put_pixel(0, 0, Rgba([255, 255, 255, 255]));
        assert_eq!(average_rgb8(&frame, (0, 0, 1, 1)), [63, 63, 63]);
    }

    #[test]
    fn single_pixel_region_returns_that_pixel() {
        let mut frame = RgbaImage::new(2, 2);
        frame.put_pixel(1, 1, Rgba([10, 20, 30, 255]));
        assert_eq!(average_rgb8(&frame, (1, 1, 1, 1)), [10, 20, 30]);
    }
}
