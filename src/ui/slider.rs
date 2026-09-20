use eframe::egui;

const GRADIENT_STEPS: usize = 48;
const HANDLE_WIDTH: f32 = 2.0;
const HANDLE_INSET: f32 = 3.0;
const BORDER_WIDTH: f32 = 1.0;
const LABEL_GAP: f32 = 2.0;
const LABEL_HEIGHT: f32 = 16.0;
const LABEL_FONT_SIZE: f32 = 14.0;
const MIN_HEIGHT: f32 = 80.0;
const LUMINANCE_RED: f32 = 0.299;
const LUMINANCE_GREEN: f32 = 0.587;
const LUMINANCE_BLUE: f32 = 0.114;
const LUMINANCE_THRESHOLD: f32 = 128.0;

pub fn column(
    ui: &mut egui::Ui,
    label: &str,
    label_color: egui::Color32,
    value: &mut f32,
    width: f32,
    gradient: impl Fn(f32) -> egui::Color32,
) -> egui::Response {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = LABEL_GAP;
        let (label_rect, _) =
            ui.allocate_exact_size(egui::vec2(width, LABEL_HEIGHT), egui::Sense::hover());
        ui.painter().text(
            label_rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(LABEL_FONT_SIZE),
            label_color,
        );
        let height = ui.available_height().max(MIN_HEIGHT);
        vertical(ui, egui::vec2(width, height), value, label_color, gradient)
    })
    .inner
}

pub fn vertical(
    ui: &mut egui::Ui,
    size: egui::Vec2,
    value: &mut f32,
    border_color: egui::Color32,
    gradient: impl Fn(f32) -> egui::Color32,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());

    if let Some(pointer) = response.interact_pointer_pos() {
        let fraction = ((pointer.y - rect.top()) / rect.height()).clamp(0.0, 1.0);
        *value = 1.0 - fraction;
        ui.ctx().request_repaint();
    }

    let painter = ui.painter().with_clip_rect(rect);
    paint_gradient(&painter, rect, &gradient);
    paint_handle(&painter, rect, *value, &gradient);
    ui.painter().rect_stroke(
        rect,
        egui::CornerRadius::ZERO,
        egui::Stroke::new(BORDER_WIDTH, border_color),
        egui::StrokeKind::Inside,
    );

    response
}

fn paint_gradient(
    painter: &egui::Painter,
    rect: egui::Rect,
    gradient: &impl Fn(f32) -> egui::Color32,
) {
    let mut mesh = egui::Mesh::default();
    for step in 0..GRADIENT_STEPS {
        let fraction = step as f32 / (GRADIENT_STEPS - 1) as f32;
        let value = 1.0 - fraction;
        let color = gradient(value);
        let y = rect.top() + fraction * rect.height();
        mesh.colored_vertex(egui::pos2(rect.left(), y), color);
        mesh.colored_vertex(egui::pos2(rect.right(), y), color);
    }
    for step in 0..(GRADIENT_STEPS as u32 - 1) {
        let top_left = step * 2;
        let top_right = top_left + 1;
        let bottom_left = top_left + 2;
        let bottom_right = top_left + 3;
        mesh.add_triangle(top_left, top_right, bottom_left);
        mesh.add_triangle(top_right, bottom_right, bottom_left);
    }
    painter.add(egui::Shape::mesh(mesh));
}

fn paint_handle(
    painter: &egui::Painter,
    rect: egui::Rect,
    value: f32,
    gradient: &impl Fn(f32) -> egui::Color32,
) {
    let y = rect.top() + (1.0 - value) * rect.height();
    painter.line_segment(
        [
            egui::pos2(rect.left() + HANDLE_INSET, y),
            egui::pos2(rect.right() - HANDLE_INSET, y),
        ],
        egui::Stroke::new(HANDLE_WIDTH, contrasting_color(gradient(value))),
    );
}

fn contrasting_color(color: egui::Color32) -> egui::Color32 {
    let luminance = LUMINANCE_RED * f32::from(color.r())
        + LUMINANCE_GREEN * f32::from(color.g())
        + LUMINANCE_BLUE * f32::from(color.b());
    if luminance > LUMINANCE_THRESHOLD {
        egui::Color32::BLACK
    } else {
        egui::Color32::WHITE
    }
}
