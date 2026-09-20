use image::RgbaImage;
use xcap::Monitor;

use crate::capture::{CaptureResult, DesktopRect, composite};

pub fn capture() -> CaptureResult<(RgbaImage, DesktopRect)> {
    let monitors = Monitor::all()?;
    match monitors.as_slice() {
        [monitor] => capture_monitor(monitor),
        _ => composite_monitors(&monitors),
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

fn composite_monitors(monitors: &[Monitor]) -> CaptureResult<(RgbaImage, DesktopRect)> {
    let mut captures = Vec::with_capacity(monitors.len());
    for monitor in monitors {
        let image = monitor.capture_image()?;
        captures.push((image, monitor.x()?, monitor.y()?));
    }
    composite(captures)
}
