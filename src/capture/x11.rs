use image::{RgbaImage, imageops};
use xcap::Monitor;

use crate::capture::{CaptureBackend, CaptureResult, DesktopCapture, DesktopRect};

pub struct X11Backend;

impl CaptureBackend for X11Backend {
    fn capture_fullscreen(&self) -> CaptureResult<DesktopCapture> {
        let monitors = Monitor::all()?;
        match monitors.as_slice() {
            [monitor] => {
                let image = monitor.capture_image()?;
                let rect = DesktopRect {
                    x: monitor.x()?,
                    y: monitor.y()?,
                    width: image.width(),
                    height: image.height(),
                };
                Ok(DesktopCapture { image, rect })
            }
            _ => composite(&monitors),
        }
    }
}

fn composite(monitors: &[Monitor]) -> CaptureResult<DesktopCapture> {
    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;

    for monitor in monitors {
        let x = monitor.x()?;
        let y = monitor.y()?;
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x + monitor.width()? as i32);
        max_y = max_y.max(y + monitor.height()? as i32);
    }

    let width = (max_x - min_x).max(0) as u32;
    let height = (max_y - min_y).max(0) as u32;
    let mut canvas = RgbaImage::new(width, height);

    for monitor in monitors {
        let image = monitor.capture_image()?;
        let x = i64::from(monitor.x()? - min_x);
        let y = i64::from(monitor.y()? - min_y);
        imageops::overlay(&mut canvas, &image, x, y);
    }

    Ok(DesktopCapture {
        image: canvas,
        rect: DesktopRect {
            x: min_x,
            y: min_y,
            width,
            height,
        },
    })
}
