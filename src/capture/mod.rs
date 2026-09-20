pub mod wayland;
pub mod x11;

use image::RgbaImage;

#[derive(Debug, thiserror::Error)]
pub enum CaptureError {
    #[error("capture backend is not implemented: {0}")]
    NotImplemented(&'static str),
    #[error("failed to create the portal runtime")]
    Runtime(#[source] std::io::Error),
    #[error("portal screenshot request failed")]
    Portal(#[from] ashpd::Error),
    #[error("failed to read the screenshot file")]
    Io(#[from] std::io::Error),
    #[error("failed to decode the screenshot")]
    Image(#[from] image::ImageError),
    #[error("screenshot URI is not a valid file path: {0}")]
    InvalidUri(String),
}

pub type CaptureResult<T> = Result<T, CaptureError>;

pub trait CaptureBackend {
    fn capture_fullscreen(&self) -> CaptureResult<RgbaImage>;

    fn uses_static_frame(&self) -> bool {
        false
    }
}

pub fn detect() -> CaptureResult<Box<dyn CaptureBackend>> {
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        Ok(Box::new(wayland::WaylandBackend::new()?))
    } else {
        Ok(Box::new(x11::X11Backend))
    }
}
