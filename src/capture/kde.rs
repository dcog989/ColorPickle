use std::collections::HashMap;
use std::io::Read;

use image::RgbaImage;
use zbus::blocking::{Connection, Proxy};
use zbus::zvariant::{DynamicTuple, Fd, OwnedValue};

use crate::capture::{CaptureBackend, CaptureError, CaptureResult};

const SERVICE: &str = "org.kde.KWin.ScreenShot2";
const PATH: &str = "/org/kde/KWin/ScreenShot2";
const INTERFACE: &str = "org.kde.KWin.ScreenShot2";
const METHOD_WORKSPACE: &str = "CaptureWorkspace";

const OPTION_NATIVE_RESOLUTION: &str = "native-resolution";
const KEY_WIDTH: &str = "width";
const KEY_HEIGHT: &str = "height";
const KEY_STRIDE: &str = "stride";
const KEY_FORMAT: &str = "format";

const BYTES_PER_PIXEL: usize = 4;
const OPAQUE: u8 = 255;

const FORMAT_RGB32: u32 = 4;
const FORMAT_ARGB32: u32 = 5;
const FORMAT_ARGB32_PREMULTIPLIED: u32 = 6;
const FORMAT_RGBX8888: u32 = 16;
const FORMAT_RGBA8888: u32 = 17;
const FORMAT_RGBA8888_PREMULTIPLIED: u32 = 18;

pub struct KdeBackend {
    frame: RgbaImage,
}

impl KdeBackend {
    pub fn new() -> CaptureResult<Self> {
        Ok(Self {
            frame: capture_workspace()?,
        })
    }
}

impl CaptureBackend for KdeBackend {
    fn capture_fullscreen(&self) -> CaptureResult<RgbaImage> {
        Ok(self.frame.clone())
    }
}

fn capture_workspace() -> CaptureResult<RgbaImage> {
    tracing::debug!("kwin: connecting to ScreenShot2");
    let connection = Connection::session()?;
    let proxy = Proxy::new(&connection, SERVICE, PATH, INTERFACE)?;

    let mut options: HashMap<String, OwnedValue> = HashMap::new();
    options.insert(OPTION_NATIVE_RESOLUTION.to_owned(), OwnedValue::from(true));

    let (mut reader, writer) = std::io::pipe()?;
    let body = DynamicTuple((&options, Fd::from(&writer)));
    let reply: HashMap<String, OwnedValue> = proxy.call(METHOD_WORKSPACE, &body)?;
    drop(body);
    drop(writer);

    let width = get_u32(&reply, KEY_WIDTH)?;
    let height = get_u32(&reply, KEY_HEIGHT)?;
    let stride = get_u32(&reply, KEY_STRIDE)?;
    let format = get_u32(&reply, KEY_FORMAT)?;

    tracing::info!(width, height, stride, format, "kwin: captured workspace");
    let byte_len = stride as usize * height as usize;
    tracing::debug!(byte_len, "kwin: reading pixels from pipe");
    let mut buffer = vec![0u8; byte_len];
    reader.read_exact(&mut buffer)?;
    tracing::debug!("kwin: pixel read complete");

    repack(&buffer, width, height, stride, format)
}

fn get_u32(map: &HashMap<String, OwnedValue>, key: &str) -> CaptureResult<u32> {
    map.get(key)
        .and_then(|value| value.downcast_ref::<u32>().ok())
        .ok_or_else(|| CaptureError::MalformedReply(key.to_owned()))
}

#[derive(Clone, Copy)]
enum PixelOrder {
    Rgba,
    Bgra,
}

impl PixelOrder {
    fn from_qimage_format(format: u32) -> CaptureResult<Self> {
        match format {
            FORMAT_RGBX8888 | FORMAT_RGBA8888 | FORMAT_RGBA8888_PREMULTIPLIED => Ok(Self::Rgba),
            FORMAT_RGB32 | FORMAT_ARGB32 | FORMAT_ARGB32_PREMULTIPLIED => Ok(Self::Bgra),
            other => Err(CaptureError::UnsupportedFormat(other)),
        }
    }

    fn rgb(self, pixel: &[u8]) -> [u8; 3] {
        match self {
            Self::Rgba => [pixel[0], pixel[1], pixel[2]],
            Self::Bgra => [pixel[2], pixel[1], pixel[0]],
        }
    }
}

fn repack(
    buffer: &[u8],
    width: u32,
    height: u32,
    stride: u32,
    format: u32,
) -> CaptureResult<RgbaImage> {
    let order = PixelOrder::from_qimage_format(format)?;
    let stride = stride as usize;
    let row_bytes = width as usize * BYTES_PER_PIXEL;
    if stride < row_bytes || buffer.len() < stride * height as usize {
        return Err(CaptureError::MalformedReply("image size".to_owned()));
    }

    let mut rgba = Vec::with_capacity(width as usize * height as usize * BYTES_PER_PIXEL);
    for row in buffer.chunks_exact(stride).take(height as usize) {
        for pixel in row[..row_bytes].chunks_exact(BYTES_PER_PIXEL) {
            let [red, green, blue] = order.rgb(pixel);
            rgba.extend_from_slice(&[red, green, blue, OPAQUE]);
        }
    }

    RgbaImage::from_raw(width, height, rgba)
        .ok_or_else(|| CaptureError::MalformedReply("image dimensions".to_owned()))
}

#[cfg(test)]
mod tests {
    use super::repack;
    use image::Rgba;

    #[test]
    fn repacks_rgba_rows_with_stride_padding() {
        let buffer = vec![
            10, 20, 30, 255, 40, 50, 60, 255, 0, 0, 0, 0, // 2 px + 4 pad
        ];
        let image = repack(&buffer, 2, 1, 12, 16).unwrap();
        assert_eq!(image.get_pixel(0, 0), &Rgba([10, 20, 30, 255]));
        assert_eq!(image.get_pixel(1, 0), &Rgba([40, 50, 60, 255]));
    }

    #[test]
    fn repacks_bgra_pixels() {
        let buffer = vec![30, 20, 10, 255];
        let image = repack(&buffer, 1, 1, 4, 6).unwrap();
        assert_eq!(image.get_pixel(0, 0), &Rgba([10, 20, 30, 255]));
    }

    #[test]
    fn rejects_unknown_format() {
        assert!(repack(&[0, 0, 0, 0], 1, 1, 4, 99).is_err());
    }
}
