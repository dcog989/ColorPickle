pub mod ext_image;
pub mod kde;
pub mod wayland;
pub mod x11;

use image::{RgbaImage, imageops};

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
    #[error("captured an empty frame")]
    EmptyFrame,
    #[error("no capture backend is available")]
    NoBackend,
}

pub type CaptureResult<T> = Result<T, CaptureError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DesktopRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl DesktopRect {
    pub fn from_image(image: &RgbaImage) -> Self {
        Self {
            x: 0,
            y: 0,
            width: image.width(),
            height: image.height(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureSource {
    KWin,
    ExtImage,
    X11,
    Portal,
}

impl CaptureSource {
    pub fn uses_portal_fallback(self) -> bool {
        matches!(self, Self::Portal)
    }
}

#[derive(Debug)]
pub struct Capture {
    pub image: RgbaImage,
    pub rect: DesktopRect,
    pub source: CaptureSource,
}

type BackendCapture = (RgbaImage, DesktopRect);

struct Backend {
    source: CaptureSource,
    available: fn() -> bool,
    run: fn() -> CaptureResult<BackendCapture>,
}

const BACKENDS: &[Backend] = &[
    Backend {
        source: CaptureSource::KWin,
        available: is_kde,
        run: kde::capture,
    },
    Backend {
        source: CaptureSource::ExtImage,
        available: is_wayland,
        run: ext_image::capture,
    },
    Backend {
        source: CaptureSource::X11,
        available: is_x11,
        run: x11::capture,
    },
    Backend {
        source: CaptureSource::Portal,
        available: is_wayland,
        run: wayland::capture,
    },
];

pub fn capture() -> CaptureResult<Capture> {
    capture_with(|| {})
}

pub fn capture_with(progress: impl FnOnce()) -> CaptureResult<Capture> {
    let mut last_error = None;
    let mut progress = Some(progress);
    for backend in BACKENDS {
        if !(backend.available)() {
            continue;
        }
        if backend.source.uses_portal_fallback()
            && let Some(notify) = progress.take()
        {
            notify();
        }
        match (backend.run)() {
            Ok((image, rect)) => {
                if image.width() == 0 || image.height() == 0 {
                    return Err(CaptureError::EmptyFrame);
                }
                tracing::info!(source = ?backend.source, "capture: frame ready");
                return Ok(Capture {
                    image,
                    rect,
                    source: backend.source,
                });
            }
            Err(error) => {
                tracing::warn!(
                    ?error,
                    source = ?backend.source,
                    "capture: backend failed; trying the next"
                );
                last_error = Some(error);
            }
        }
    }
    Err(last_error.unwrap_or(CaptureError::NoBackend))
}

pub fn composite(captures: Vec<(RgbaImage, i32, i32)>) -> CaptureResult<(RgbaImage, DesktopRect)> {
    let min_x = captures.iter().map(|(_, x, _)| *x).min().unwrap_or(0);
    let min_y = captures.iter().map(|(_, _, y)| *y).min().unwrap_or(0);
    let max_x = captures
        .iter()
        .map(|(image, x, _)| x + image.width() as i32)
        .max()
        .unwrap_or(0);
    let max_y = captures
        .iter()
        .map(|(image, _, y)| y + image.height() as i32)
        .max()
        .unwrap_or(0);

    let width = (max_x - min_x).max(0) as u32;
    let height = (max_y - min_y).max(0) as u32;
    if width == 0 || height == 0 {
        return Err(CaptureError::EmptyFrame);
    }

    let mut canvas = RgbaImage::new(width, height);
    for (image, x, y) in captures {
        imageops::overlay(
            &mut canvas,
            &image,
            i64::from(x - min_x),
            i64::from(y - min_y),
        );
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

fn is_kde() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP")
        .map(|value| value.to_ascii_lowercase().contains("kde"))
        .unwrap_or(false)
}

fn is_wayland() -> bool {
    std::env::var_os("WAYLAND_DISPLAY").is_some()
}

fn is_x11() -> bool {
    !is_wayland()
}
