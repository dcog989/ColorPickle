use eframe::egui;

const SLIDER_CORNER_RADIUS: u8 = 6;
const HANDLE_RADIUS: f32 = 6.0;
const HANDLE_INSET: f32 = 3.0;
const HANDLE_BORDER_WIDTH: f32 = 2.0;
const BORDER_WIDTH: f32 = 1.0;
const LABEL_GAP: f32 = 2.0;
const LABEL_HEIGHT: f32 = 16.0;
const LABEL_FONT_SIZE: f32 = 14.0;
const MIN_HEIGHT: f32 = 80.0;
const GRADIENT_STOPS: usize = 64;

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

    let corner = egui::CornerRadius::same(SLIDER_CORNER_RADIUS);
    let painter = ui.painter().with_clip_rect(rect);
    paint_gradient(&painter, rect, f32::from(SLIDER_CORNER_RADIUS), &gradient);
    paint_handle(&painter, rect, *value, border_color);
    ui.painter().rect_stroke(
        rect,
        corner,
        egui::Stroke::new(BORDER_WIDTH, border_color),
        egui::StrokeKind::Inside,
    );

    response
}

fn paint_gradient(
    painter: &egui::Painter,
    rect: egui::Rect,
    radius: f32,
    gradient: &impl Fn(f32) -> egui::Color32,
) {
    let stops: Vec<egui::Color32> = (0..=GRADIENT_STOPS)
        .map(|index| gradient(index as f32 / GRADIENT_STOPS as f32))
        .collect();

    let rows = (rect.height().ceil() as usize).max(2);
    let mut mesh = egui::Mesh::default();
    for index in 0..=rows {
        let fraction = index as f32 / rows as f32;
        let y = rect.top() + fraction * rect.height();
        let inset = corner_inset(radius, y - rect.top(), rect.bottom() - y);
        let color = interpolate_stops(&stops, 1.0 - fraction);
        mesh.colored_vertex(egui::pos2(rect.left() + inset, y), color);
        mesh.colored_vertex(egui::pos2(rect.right() - inset, y), color);
    }
    for index in 0..rows as u32 {
        let top_left = index * 2;
        let top_right = top_left + 1;
        let bottom_left = top_left + 2;
        let bottom_right = top_left + 3;
        mesh.add_triangle(top_left, top_right, bottom_left);
        mesh.add_triangle(top_right, bottom_right, bottom_left);
    }
    painter.add(egui::Shape::mesh(mesh));
}

fn interpolate_stops(stops: &[egui::Color32], value: f32) -> egui::Color32 {
    let scaled = value.clamp(0.0, 1.0) * (stops.len() - 1) as f32;
    let lower = scaled.floor() as usize;
    let upper = (lower + 1).min(stops.len() - 1);
    let t = scaled - lower as f32;
    let from = stops[lower];
    let to = stops[upper];
    egui::Color32::from_rgba_premultiplied(
        interpolate_u8(from.r(), to.r(), t),
        interpolate_u8(from.g(), to.g(), t),
        interpolate_u8(from.b(), to.b(), t),
        interpolate_u8(from.a(), to.a(), t),
    )
}

fn interpolate_u8(from: u8, to: u8, t: f32) -> u8 {
    (f32::from(from) + (f32::from(to) - f32::from(from)) * t).round() as u8
}

fn corner_inset(radius: f32, distance_from_top: f32, distance_from_bottom: f32) -> f32 {
    let distance = distance_from_top.min(distance_from_bottom);
    if distance >= radius {
        return 0.0;
    }
    radius - (radius * radius - (radius - distance).powi(2)).sqrt()
}

fn paint_handle(
    painter: &egui::Painter,
    rect: egui::Rect,
    value: f32,
    border_color: egui::Color32,
) {
    let y = rect.top() + (1.0 - value) * rect.height();
    let half_extent = HANDLE_RADIUS + HANDLE_BORDER_WIDTH / 2.0;
    let center = egui::pos2(
        rect.center().x,
        y.clamp(rect.top() + half_extent, rect.bottom() - half_extent),
    );
    let radius = egui::vec2((rect.width() / 2.0 - HANDLE_INSET).max(1.0), HANDLE_RADIUS);
    painter.add(egui::epaint::EllipseShape::stroke(
        center,
        radius,
        egui::Stroke::new(HANDLE_BORDER_WIDTH, border_color),
    ));
}
