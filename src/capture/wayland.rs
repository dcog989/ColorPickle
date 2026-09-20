use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;

use image::RgbaImage;

use crate::capture::{CaptureBackend, CaptureError, CaptureResult, DesktopCapture, DesktopRect};

const FILE_SCHEME: &str = "file://";
const HEX_RADIX: u32 = 16;

pub struct WaylandBackend;

impl WaylandBackend {
    pub fn new() -> CaptureResult<Self> {
        Ok(Self)
    }
}

impl CaptureBackend for WaylandBackend {
    fn capture_fullscreen(self: Box<Self>) -> CaptureResult<DesktopCapture> {
        let frame = capture_once()?;
        Ok(DesktopCapture {
            rect: DesktopRect::from_image(&frame),
            image: frame,
        })
    }

    fn uses_portal_fallback(&self) -> bool {
        true
    }
}

fn capture_once() -> CaptureResult<RgbaImage> {
    let path = block_on(take_screenshot())??;
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

fn block_on<F: std::future::Future>(future: F) -> CaptureResult<F::Output> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .map_err(CaptureError::Runtime)?;
    Ok(runtime.block_on(future))
}

fn uri_to_path(uri: &str) -> CaptureResult<PathBuf> {
    let encoded = uri
        .strip_prefix(FILE_SCHEME)
        .ok_or_else(|| CaptureError::InvalidUri(uri.to_string()))?;
    let bytes = percent_decode(encoded).ok_or_else(|| CaptureError::InvalidUri(uri.to_string()))?;
    Ok(PathBuf::from(OsString::from_vec(bytes)))
}

fn percent_decode(input: &str) -> Option<Vec<u8>> {
    let bytes = input.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let digits = bytes.get(index + 1..index + 3)?;
            if !digits.iter().all(u8::is_ascii_hexdigit) {
                return None;
            }
            let value = u8::from_str_radix(std::str::from_utf8(digits).ok()?, HEX_RADIX).ok()?;
            output.push(value);
            index += 3;
        } else {
            output.push(bytes[index]);
            index += 1;
        }
    }
    Some(output)
}

#[cfg(test)]
mod tests {
    use super::uri_to_path;
    use std::path::PathBuf;

    #[test]
    fn decodes_plain_file_uri() {
        assert_eq!(
            uri_to_path("file:///tmp/shot.png").unwrap(),
            PathBuf::from("/tmp/shot.png")
        );
    }

    #[test]
    fn decodes_percent_escapes() {
        assert_eq!(
            uri_to_path("file:///tmp/a%20b%2Bc.png").unwrap(),
            PathBuf::from("/tmp/a b+c.png")
        );
    }

    #[test]
    fn rejects_non_file_uri() {
        assert!(uri_to_path("https://example.com/x.png").is_err());
    }

    #[test]
    fn rejects_truncated_escape() {
        assert!(uri_to_path("file:///tmp/a%2.png").is_err());
    }
}
