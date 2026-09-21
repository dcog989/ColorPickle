use image::{Rgba, RgbaImage};
use x11rb::connection::Connection;
use x11rb::image::{Image, PixelLayout};
use x11rb::protocol::randr;
use x11rb::protocol::xproto::{self, Window};

use crate::capture::{CaptureError, CaptureResult, Point};

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

pub fn capture(cursor: Option<Point>) -> CaptureResult<RgbaImage> {
    let (connection, screen_num) = x11rb::connect(None)?;
    let screen = &connection.setup().roots[screen_num];
    let root = screen.root;
    let (x, y, width, height) = match cursor.and_then(|point| monitor_at(&connection, root, point))
    {
        Some(rect) => rect,
        None => (0, 0, screen.width_in_pixels, screen.height_in_pixels),
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
    for pixel_y in 0..height {
        for pixel_x in 0..width {
            let (red, green, blue) = layout.decode(image.get_pixel(pixel_x, pixel_y));
            canvas.put_pixel(
                u32::from(pixel_x),
                u32::from(pixel_y),
                Rgba([high_byte(red), high_byte(green), high_byte(blue), 255]),
            );
        }
    }

    Ok(canvas)
}

fn monitor_at(
    connection: &impl Connection,
    root: Window,
    point: Point,
) -> Option<(i16, i16, u16, u16)> {
    let reply = randr::get_monitors(connection, root, true)
        .ok()?
        .reply()
        .ok()?;
    reply
        .monitors
        .iter()
        .find(|monitor| {
            point.x >= i32::from(monitor.x)
                && point.x < i32::from(monitor.x) + i32::from(monitor.width)
                && point.y >= i32::from(monitor.y)
                && point.y < i32::from(monitor.y) + i32::from(monitor.height)
        })
        .map(|monitor| (monitor.x, monitor.y, monitor.width, monitor.height))
}

fn high_byte(component: u16) -> u8 {
    (component >> 8) as u8
}
