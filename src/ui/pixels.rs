use eframe::egui;

pub fn pixel_at(image: &egui::ColorImage, uv: egui::Pos2) -> (u32, u32) {
    let width = image.size[0] as f32;
    let height = image.size[1] as f32;
    let x = (uv.x * width).clamp(0.0, width - 1.0);
    let y = (uv.y * height).clamp(0.0, height - 1.0);
    (x as u32, y as u32)
}

pub fn pixel_bounds(
    image: &egui::ColorImage,
    a: egui::Pos2,
    b: egui::Pos2,
) -> (u32, u32, u32, u32) {
    let (x0, y0) = pixel_at(image, a);
    let (x1, y1) = pixel_at(image, b);
    (x0.min(x1), y0.min(y1), x0.max(x1), y0.max(y1))
}

pub fn average_rgb8(image: &egui::ColorImage, bounds: (u32, u32, u32, u32)) -> [u8; 3] {
    let width = image.size[0];
    let (start_x, start_y, end_x, end_y) = bounds;
    let mut sums = [0u64; 3];
    let mut count = 0u64;
    for y in start_y..=end_y {
        for x in start_x..=end_x {
            let pixel = image.pixels[y as usize * width + x as usize];
            sums[0] += u64::from(pixel.r());
            sums[1] += u64::from(pixel.g());
            sums[2] += u64::from(pixel.b());
            count += 1;
        }
    }

    if count == 0 {
        return [0, 0, 0];
    }
    [
        (sums[0] / count) as u8,
        (sums[1] / count) as u8,
        (sums[2] / count) as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::average_rgb8;
    use eframe::egui::{self, Color32};

    #[test]
    fn averages_a_region_block() {
        let mut pixels = vec![Color32::TRANSPARENT; 4 * 4];
        for y in 0..2 {
            for x in 0..2 {
                pixels[y * 4 + x] = Color32::RED;
            }
        }
        let image = egui::ColorImage::new([4, 4], pixels);
        assert_eq!(average_rgb8(&image, (0, 0, 1, 1)), [255, 0, 0]);
    }

    #[test]
    fn averages_partial_coverage() {
        let mut pixels = vec![Color32::TRANSPARENT; 2 * 2];
        pixels[0] = Color32::WHITE;
        let image = egui::ColorImage::new([2, 2], pixels);
        assert_eq!(average_rgb8(&image, (0, 0, 1, 1)), [63, 63, 63]);
    }

    #[test]
    fn single_pixel_region_returns_that_pixel() {
        let mut pixels = vec![Color32::TRANSPARENT; 2 * 2];
        pixels[3] = Color32::from_rgb(10, 20, 30);
        let image = egui::ColorImage::new([2, 2], pixels);
        assert_eq!(average_rgb8(&image, (1, 1, 1, 1)), [10, 20, 30]);
    }
}
