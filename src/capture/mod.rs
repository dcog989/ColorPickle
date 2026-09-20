pub mod ext_image;
pub mod kde;
pub mod wayland;
pub mod x11;

use image::RgbaImage;

#[derive(Debug, thiserror::Error)]
pub enum CaptureError {
    #[error("X11 capture failed")]
    Xcap(#[from] xcap::XCapError),
    #[error("failed to create the portal runtime")]
    Runtime(#[source] std::io::Error),
    #[error("portal screenshot request failed")]
    Portal(#[from] ashpd::Error),
    #[error("KWin screen shot request failed")]
    Dbus(#[from] zbus::Error),
    #[error("KWin screen shot reply is missing field: {0}")]
    MalformedReply(String),
    #[error("unsupported QImage format: {0}")]
    UnsupportedFormat(u32),
    #[error("failed to read capture data")]
    Io(#[from] std::io::Error),
    #[error("failed to decode the screenshot")]
    Image(#[from] image::ImageError),
    #[error("screenshot URI is not a valid file path: {0}")]
    InvalidUri(String),
    #[error("ext-image-copy-capture is not available")]
    ExtImageUnavailable,
    #[error("ext-image-copy-capture failed: {0}")]
    ExtImage(String),
}

pub type CaptureResult<T> = Result<T, CaptureError>;

pub trait CaptureBackend {
    fn capture_fullscreen(&self) -> CaptureResult<RgbaImage>;

    fn uses_portal_fallback(&self) -> bool {
        false
    }
}

pub fn detect() -> CaptureResult<Box<dyn CaptureBackend>> {
    if is_kde() {
        match kde::KdeBackend::new() {
            Ok(backend) => {
                tracing::info!("capture: using KWin ScreenShot2");
                return Ok(Box::new(backend));
            }
            Err(error) => {
                tracing::warn!(
                    ?error,
                    "KWin ScreenShot2 capture failed; trying the next backend"
                );
            }
        }
    }
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        match ext_image::ExtImageBackend::new() {
            Ok(backend) => {
                tracing::info!("capture: using ext-image-copy-capture");
                return Ok(Box::new(backend));
            }
            Err(error) => {
                tracing::warn!(
                    ?error,
                    "ext-image-copy-capture unavailable; using the portal"
                );
            }
        }
        tracing::info!("capture: using xdg-desktop-portal");
        Ok(Box::new(wayland::WaylandBackend::new()?))
    } else {
        tracing::info!("capture: using X11 xcap");
        Ok(Box::new(x11::X11Backend))
    }
}

fn is_kde() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP")
        .map(|value| value.to_ascii_lowercase().contains("kde"))
        .unwrap_or(false)
}
