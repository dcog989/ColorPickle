use image::{RgbaImage, imageops};
use xcap::Monitor;

use crate::capture::{CaptureResult, DesktopRect};

pub fn capture() -> CaptureResult<(RgbaImage, DesktopRect)> {
    let monitors = Monitor::all()?;
    match monitors.as_slice() {
        [monitor] => capture_monitor(monitor),
        _ => composite(&monitors),
    }
}

fn capture_monitor(monitor: &Monitor) -> CaptureResult<(RgbaImage, DesktopRect)> {
    let image = monitor.capture_image()?;
    let rect = DesktopRect {
        x: monitor.x()?,
        y: monitor.y()?,
        width: image.width(),
        height: image.height(),
    };
    Ok((image, rect))
}

fn composite(monitors: &[Monitor]) -> CaptureResult<(RgbaImage, DesktopRect)> {
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

    let width = max_x.saturating_sub(min_x).max(0) as u32;
    let height = max_y.saturating_sub(min_y).max(0) as u32;
    let mut canvas = RgbaImage::new(width, height);

    for monitor in monitors {
        let image = monitor.capture_image()?;
        let x = i64::from(monitor.x()? - min_x);
        let y = i64::from(monitor.y()? - min_y);
        imageops::overlay(&mut canvas, &image, x, y);
    }

    Ok((
        canvas,
        DesktopRect {
            x: min_x,
            y: min_y,
            width,
            height,
        },
    ))
}
