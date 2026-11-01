use std::path::{Path, PathBuf};

use image::RgbaImage;
use percent_encoding::percent_decode_str;

use crate::capture::x11::{self, MonitorBounds};
use crate::capture::{CaptureError, CaptureResult, OutputInfo, Point};

const FILE_URI_PREFIX: &str = "file://";

pub fn capture(cursor: Option<Point>) -> CaptureResult<(RgbaImage, OutputInfo)> {
    let image = capture_once()?;
    Ok(match cursor.and_then(x11::monitor_bounds) {
        Some(bounds) => crop_to_monitor(image, bounds),
        None => {
            tracing::debug!("portal: no monitor under the pointer; keeping the whole desktop");
            (image, OutputInfo::default())
        }
    })
}

/// The portal returns the whole desktop, but the overlay covers one monitor.
/// Crop the image to the monitor under the pointer so the two line up. The
/// rect is scaled from XWayland's layout into the image.
///
/// shortcut: one scale factor for the whole desktop, so a setup that mixes
/// scale factors crops the wrong region; upgrade when the portal reports
/// per-output geometry.
fn crop_to_monitor(image: RgbaImage, bounds: MonitorBounds) -> (RgbaImage, OutputInfo) {
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
        tracing::warn!(
            rect = ?bounds.rect,
            "portal: the monitor crop is empty; keeping the whole desktop"
        );
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
    let path = pollster::block_on(take_screenshot())?;
    let image = read_image(&path);
    remove_screenshot(&path);
    image
}

/// Removes a screenshot the portal left in temporary storage. A reply naming
/// any other path is not ours to delete.
fn remove_screenshot(path: &Path) {
    if !is_temporary(path) {
        tracing::debug!(path = %path.display(), "portal: leaving a screenshot outside temporary storage");
        return;
    }
    if let Err(error) = std::fs::remove_file(path) {
        tracing::warn!(?error, path = ?path, "failed to remove the portal screenshot");
    }
}

fn is_temporary(path: &Path) -> bool {
    let configured = ["XDG_RUNTIME_DIR", "XDG_CACHE_HOME"]
        .iter()
        .filter_map(|name| std::env::var_os(name).map(PathBuf::from));
    std::iter::once(std::env::temp_dir())
        .chain(configured)
        .any(|root| path.starts_with(root))
}

/// The portal does not guarantee a `.png` suffix, so the format is guessed from
/// the bytes. Only the `png` decoder is enabled, so guessing is exact.
fn read_image(path: &Path) -> CaptureResult<RgbaImage> {
    let reader = image::ImageReader::open(path)?.with_guessed_format()?;
    Ok(reader.decode()?.into_rgba8())
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

fn uri_to_path(uri: &str) -> CaptureResult<PathBuf> {
    // A `file://` URI with a non-empty authority (`file://host/...`) is not a
    // local path, so the remainder has to be absolute.
    let decoded = uri
        .strip_prefix(FILE_URI_PREFIX)
        .filter(|path| path.starts_with('/'))
        .and_then(|path| percent_decode_str(path).decode_utf8().ok())
        .ok_or_else(|| CaptureError::InvalidUri(uri.to_string()))?;
    Ok(PathBuf::from(decoded.as_ref()))
}

#[cfg(test)]
mod tests {
    use super::{crop_to_monitor, is_temporary, uri_to_path};
    use crate::capture::OutputInfo;
    use crate::capture::x11::MonitorBounds;
    use image::{Rgba, RgbaImage};
    use std::path::{Path, PathBuf};

    const RED: Rgba<u8> = Rgba([255, 0, 0, 255]);
    const GREEN: Rgba<u8> = Rgba([0, 255, 0, 255]);

    fn bounds(rect: (i32, i32, u32, u32), desktop: (u32, u32)) -> MonitorBounds {
        MonitorBounds {
            rect,
            desktop,
            output: OutputInfo {
                name: Some("DP-1".to_owned()),
                position: Some((rect.0, rect.1)),
            },
        }
    }

    /// Left half red, right half green.
    fn split_image(width: u32, height: u32) -> RgbaImage {
        RgbaImage::from_fn(
            width,
            height,
            |x, _| {
                if x < width / 2 { RED } else { GREEN }
            },
        )
    }

    #[test]
    fn keeps_a_single_monitor_image_whole() {
        let image = split_image(200, 100);
        let (cropped, output) = crop_to_monitor(image, bounds((0, 0, 200, 100), (200, 100)));

        assert_eq!(cropped.dimensions(), (200, 100));
        assert_eq!(output.name.as_deref(), Some("DP-1"));
        assert_eq!(output.position, Some((0, 0)));
    }

    #[test]
    fn crops_the_right_monitor_out_of_a_scaled_desktop() {
        // Two 200x100 monitors side by side at 2x, so the image is 800x200.
        let image = split_image(800, 200);
        let (cropped, _) = crop_to_monitor(image, bounds((200, 0, 200, 100), (400, 100)));

        assert_eq!(cropped.dimensions(), (400, 200));
        assert!(cropped.pixels().all(|pixel| *pixel == GREEN));
    }

    #[test]
    fn keeps_the_whole_image_when_the_crop_is_empty() {
        let image = split_image(200, 100);
        let (cropped, output) = crop_to_monitor(image, bounds((500, 0, 100, 100), (200, 100)));

        assert_eq!(cropped.dimensions(), (200, 100));
        assert_eq!(output.position, None);
    }

    #[test]
    fn clamps_a_monitor_that_hangs_off_the_desktop() {
        let image = split_image(200, 100);
        let (cropped, _) = crop_to_monitor(image, bounds((150, 0, 200, 100), (200, 100)));

        assert_eq!(cropped.dimensions(), (50, 100));
        assert!(cropped.pixels().all(|pixel| *pixel == GREEN));
    }

    #[test]
    fn screenshots_are_only_removed_from_temporary_storage() {
        assert!(is_temporary(&std::env::temp_dir().join("shot.png")));
        assert!(!is_temporary(Path::new("/etc/passwd")));
        assert!(!is_temporary(Path::new(
            "/home/somebody/.config/state.toml"
        )));
    }

    #[test]
    fn decodes_file_uris() {
        assert_eq!(
            uri_to_path("file:///tmp/My%20Shot.png").unwrap(),
            PathBuf::from("/tmp/My Shot.png")
        );
        assert_eq!(
            uri_to_path("file:///tmp/K%C3%A9k.png").unwrap(),
            PathBuf::from("/tmp/Kék.png")
        );
    }

    #[test]
    fn rejects_unusable_uris() {
        assert!(uri_to_path("https://example.com/shot.png").is_err());
        assert!(uri_to_path("file://host/tmp/shot.png").is_err());
        assert!(uri_to_path("file:///tmp/bad%ff.png").is_err());
    }
}
