use image::RgbaImage;
use x11rb::connection::Connection;
use x11rb::image::{Image, PixelLayout};
use x11rb::protocol::randr;
use x11rb::protocol::xproto::{self, Window};

use crate::capture::{CaptureError, CaptureResult, OutputInfo, Point};

const BYTES_PER_PIXEL: usize = 4;
const OPAQUE: u8 = 255;

pub fn pointer_position() -> Option<Point> {
    let (connection, screen) = x11rb::connect(None).ok()?;
    let root = connection.setup().roots.get(screen)?.root;
    let reply = xproto::query_pointer(&connection, root)
        .ok()?
        .reply()
        .ok()?;
    Some(Point {
        x: i32::from(reply.root_x),
        y: i32::from(reply.root_y),
    })
}

#[derive(Debug)]
pub struct MonitorBounds {
    pub rect: (i32, i32, u32, u32),
    pub desktop: (u32, u32),
    pub output: OutputInfo,
}

/// Monitor containing `point`, plus the whole-desktop size. Available on
/// Wayland through XWayland, whose layout mirrors the compositor's.
pub fn monitor_bounds(point: Point) -> Option<MonitorBounds> {
    let (connection, screen_num) = x11rb::connect(None).ok()?;
    let screen = connection.setup().roots.get(screen_num)?;
    let (rect, output) = monitor_at(&connection, screen.root, point)?;
    Some(MonitorBounds {
        rect: (
            i32::from(rect.0),
            i32::from(rect.1),
            u32::from(rect.2),
            u32::from(rect.3),
        ),
        desktop: (
            u32::from(screen.width_in_pixels),
            u32::from(screen.height_in_pixels),
        ),
        output,
    })
}

pub fn capture(cursor: Option<Point>) -> CaptureResult<(RgbaImage, OutputInfo)> {
    let (connection, screen_num) = x11rb::connect(None)?;
    let screen = &connection.setup().roots[screen_num];
    let root = screen.root;
    let (x, y, width, height, output) =
        match cursor.and_then(|point| monitor_at(&connection, root, point)) {
            Some((rect, output)) => (rect.0, rect.1, rect.2, rect.3, output),
            None => (
                0,
                0,
                screen.width_in_pixels,
                screen.height_in_pixels,
                OutputInfo::default(),
            ),
        };

    let (image, visual_id) = Image::get(&connection, root, x, y, width, height)?;
    let visual = screen
        .allowed_depths
        .iter()
        .flat_map(|depth| &depth.visuals)
        .find(|visual| visual.visual_id == visual_id)
        .ok_or(CaptureError::X11Visual(visual_id))?;
    let layout = PixelLayout::from_visual_type(*visual)?;

    let mut canvas = RgbaImage::new(u32::from(width), u32::from(height));
    let (pixels, _) = canvas.as_chunks_mut::<BYTES_PER_PIXEL>();
    for (row, pixel_y) in pixels.chunks_mut(usize::from(width)).zip(0..height) {
        for (pixel, pixel_x) in row.iter_mut().zip(0..width) {
            let (red, green, blue) = layout.decode(image.get_pixel(pixel_x, pixel_y));
            *pixel = [high_byte(red), high_byte(green), high_byte(blue), OPAQUE];
        }
    }

    Ok((canvas, output))
}

fn monitor_at(
    connection: &impl Connection,
    root: Window,
    point: Point,
) -> Option<((i16, i16, u16, u16), OutputInfo)> {
    let reply = randr::get_monitors(connection, root, true)
        .ok()?
        .reply()
        .ok()?;
    let monitor = reply.monitors.iter().find(|monitor| {
        point.x >= i32::from(monitor.x)
            && point.x < i32::from(monitor.x) + i32::from(monitor.width)
            && point.y >= i32::from(monitor.y)
            && point.y < i32::from(monitor.y) + i32::from(monitor.height)
    })?;
    let output = OutputInfo {
        name: atom_name(connection, monitor.name),
        position: Some((i32::from(monitor.x), i32::from(monitor.y))),
        size: Some((u32::from(monitor.width), u32::from(monitor.height))),
    };
    Some((
        (monitor.x, monitor.y, monitor.width, monitor.height),
        output,
    ))
}

fn atom_name(connection: &impl Connection, atom: xproto::Atom) -> Option<String> {
    let reply = xproto::get_atom_name(connection, atom).ok()?.reply().ok()?;
    String::from_utf8(reply.name).ok()
}

fn high_byte(component: u16) -> u8 {
    (component >> 8) as u8
}
