pub mod ext_image;
pub mod kde;
pub mod wayland;
pub mod x11;

use image::RgbaImage;

#[derive(Debug, thiserror::Error)]
pub enum CaptureError {
    #[error("X11 connection failed")]
    X11Connect(#[from] x11rb::errors::ConnectError),
    #[error("X11 capture failed")]
    X11Reply(#[from] x11rb::errors::ReplyError),
    #[error("X11 image conversion failed")]
    X11Parse(#[from] x11rb::errors::ParseError),
    #[error("X11 visual {0} is unavailable")]
    X11Visual(u32),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OutputInfo {
    pub name: Option<String>,
    pub position: Option<(i32, i32)>,
    pub size: Option<(u32, u32)>,
}

#[derive(Debug)]
pub struct Capture {
    pub image: RgbaImage,
    pub source: CaptureSource,
    pub output: OutputInfo,
}

struct Backend {
    source: CaptureSource,
    available: fn() -> bool,
    cursor: fn() -> Option<Point>,
    run: fn(Option<Point>) -> CaptureResult<(RgbaImage, OutputInfo)>,
}

const BACKENDS: &[Backend] = &[
    Backend {
        source: CaptureSource::KWin,
        available: is_kde,
        cursor: no_cursor,
        run: kde::capture,
    },
    Backend {
        source: CaptureSource::ExtImage,
        available: is_wayland,
        // Wayland exposes no global pointer, but XWayland does, and its root
        // coordinates share the compositor's logical output layout.
        cursor: x11::pointer_position,
        run: ext_image::capture,
    },
    Backend {
        source: CaptureSource::X11,
        available: is_x11,
        cursor: x11::pointer_position,
        run: x11::capture,
    },
    Backend {
        source: CaptureSource::Portal,
        available: is_wayland,
        // The portal image spans the whole desktop; the pointer selects which
        // monitor to crop it to, via XWayland as above.
        cursor: x11::pointer_position,
        run: wayland::capture,
    },
];

fn no_cursor() -> Option<Point> {
    None
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
        let cursor = (backend.cursor)();
        match (backend.run)(cursor) {
            Ok((image, output)) if image.width() > 0 && image.height() > 0 => {
                tracing::info!(source = ?backend.source, "capture: frame ready");
                return Ok(Capture {
                    image,
                    source: backend.source,
                    output,
                });
            }
            Ok(_) => {
                tracing::warn!(source = ?backend.source, "capture: empty frame; trying the next");
                last_error = Some(CaptureError::EmptyFrame);
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

const BYTES_PER_PIXEL: usize = 4;
const OPAQUE: u8 = 255;

/// Channel order of a 4-byte source pixel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelOrder {
    Rgba,
    Bgra,
}

/// Drop row padding, force opaque alpha, and swizzle a 4-byte-per-pixel buffer
/// to RGBA. `None` when `buffer` does not hold `stride * height` bytes with at
/// least `width * BYTES_PER_PIXEL` bytes per row.
pub fn compact_and_swizzle(
    mut buffer: Vec<u8>,
    width: u32,
    height: u32,
    stride: usize,
    order: PixelOrder,
) -> Option<RgbaImage> {
    let row_bytes = width as usize * BYTES_PER_PIXEL;
    if stride < row_bytes || buffer.len() < stride * height as usize {
        return None;
    }

    if stride != row_bytes {
        for row in 1..height as usize {
            let source = row * stride;
            buffer.copy_within(source..source + row_bytes, row * row_bytes);
        }
        buffer.truncate(height as usize * row_bytes);
    }

    for pixel in buffer.as_chunks_mut::<BYTES_PER_PIXEL>().0 {
        if order == PixelOrder::Bgra {
            pixel.swap(0, 2);
        }
        pixel[3] = OPAQUE;
    }

    RgbaImage::from_raw(width, height, buffer)
}
