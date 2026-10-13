use std::path::PathBuf;

use image::RgbaImage;
use url::Url;

use crate::capture::x11;
use crate::capture::{CaptureError, CaptureResult, OutputInfo, Point};

pub fn capture(cursor: Option<Point>) -> CaptureResult<(RgbaImage, OutputInfo)> {
    let image = capture_once()?;
    Ok(crop_to_monitor(image, cursor))
}

/// The portal returns the whole desktop, but the overlay covers one monitor.
/// Crop the image to the monitor under the pointer so the two line up. The
/// rect is scaled from XWayland's layout into the image, which is exact when
/// every monitor shares one scale factor.
fn crop_to_monitor(image: RgbaImage, cursor: Option<Point>) -> (RgbaImage, OutputInfo) {
    let Some(bounds) = cursor.and_then(x11::monitor_bounds) else {
        return (image, OutputInfo::default());
    };
    let (desktop_width, desktop_height) = bounds.desktop;
    if desktop_width == 0 || desktop_height == 0 {
        return (image, OutputInfo::default());
    }

    let image_width = image.width();
    let image_height = image.height();
    let scale_x = f64::from(image_width) / f64::from(desktop_width);
    let scale_y = f64::from(image_height) / f64::from(desktop_height);
    let (monitor_x, monitor_y, monitor_width, monitor_height) = bounds.rect;

    let left = (f64::from(monitor_x) * scale_x)
        .round()
        .clamp(0.0, f64::from(image_width));
    let top = (f64::from(monitor_y) * scale_y)
        .round()
        .clamp(0.0, f64::from(image_height));
    let right = (f64::from(monitor_x + monitor_width as i32) * scale_x)
        .round()
        .clamp(0.0, f64::from(image_width));
    let bottom = (f64::from(monitor_y + monitor_height as i32) * scale_y)
        .round()
        .clamp(0.0, f64::from(image_height));

    let crop_width = (right - left) as u32;
    let crop_height = (bottom - top) as u32;
    if crop_width == 0 || crop_height == 0 {
        return (image, OutputInfo::default());
    }
    if left == 0.0 && top == 0.0 && crop_width == image_width && crop_height == image_height {
        return (image, bounds.output);
    }

    let cropped =
        image::imageops::crop_imm(&image, left as u32, top as u32, crop_width, crop_height)
            .to_image();
    (cropped, bounds.output)
}

fn capture_once() -> CaptureResult<RgbaImage> {
    let path = block_on(take_screenshot())?;
    let image = image::open(&path).map(|image| image.into_rgba8());
    if let Err(error) = std::fs::remove_file(&path) {
        tracing::warn!(?error, path = ?path, "failed to remove the portal screenshot");
    }
    Ok(image?)
}

async fn take_screenshot() -> CaptureResult<PathBuf> {
    let response = ashpd::desktop::screenshot::Screenshot::request()
        .interactive(false)
        .modal(true)
        .send()
        .await?
        .response()?;
    uri_to_path(response.uri().as_str())
}

fn block_on<F: std::future::Future>(future: F) -> F::Output {
    pollster::block_on(future)
}

fn uri_to_path(uri: &str) -> CaptureResult<PathBuf> {
    Url::parse(uri)
        .ok()
        .and_then(|url| url.to_file_path().ok())
        .ok_or_else(|| CaptureError::InvalidUri(uri.to_string()))
}
