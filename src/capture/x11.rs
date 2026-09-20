use image::RgbaImage;

use crate::capture::{CaptureBackend, CaptureError, CaptureResult};

pub struct X11Backend;

impl CaptureBackend for X11Backend {
    fn capture_fullscreen(&self) -> CaptureResult<RgbaImage> {
        Err(CaptureError::NotImplemented("x11 (xcap)"))
    }
}
