use image::{Rgba, RgbaImage};
use x11rb::connection::Connection;
use x11rb::image::{Image, PixelLayout};

use crate::capture::{CaptureError, CaptureResult};

pub fn capture() -> CaptureResult<RgbaImage> {
    let (connection, screen) = x11rb::connect(None)?;
    let root = &connection.setup().roots[screen];
    let width = root.width_in_pixels;
    let height = root.height_in_pixels;

    let (image, visual_id) = Image::get(&connection, root.root, 0, 0, width, height)?;
    let visual = root
        .allowed_depths
        .iter()
        .flat_map(|depth| &depth.visuals)
        .find(|visual| visual.visual_id == visual_id)
        .ok_or(CaptureError::X11Visual(visual_id))?;
    let layout = PixelLayout::from_visual_type(*visual)?;

    let mut canvas = RgbaImage::new(u32::from(width), u32::from(height));
    for y in 0..height {
        for x in 0..width {
            let (red, green, blue) = layout.decode(image.get_pixel(x, y));
            canvas.put_pixel(
                u32::from(x),
                u32::from(y),
                Rgba([high_byte(red), high_byte(green), high_byte(blue), 255]),
            );
        }
    }

    Ok(canvas)
}

fn high_byte(component: u16) -> u8 {
    (component >> 8) as u8
}
