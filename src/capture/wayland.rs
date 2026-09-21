use std::path::PathBuf;

use image::RgbaImage;
use url::Url;

use crate::capture::{CaptureError, CaptureResult};

pub fn capture() -> CaptureResult<RgbaImage> {
    capture_once()
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
