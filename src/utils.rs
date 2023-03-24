use image::DynamicImage;
use std::{ffi::OsStr, path::Path};

pub fn is_png(path: &Path) -> bool {
    path.extension().and_then(OsStr::to_str) == Some("png")
}

pub fn closest_color(available_colors: &[[u8; 3]], target: &[u8; 3]) -> [u8; 3] {
    let mut biggest_difference: i64 = i64::max_value();
    let mut closest = [0u8, 0u8, 0u8];

    for color in available_colors.iter() {
        let distance = rgb_distance(target, color);
        if distance < biggest_difference {
            biggest_difference = distance;
            closest = *color;
        }
    }

    closest
}

pub fn rgb_distance(a: &[u8; 3], b: &[u8; 3]) -> i64 {
    let (r1, g1, b1) = (a[0] as i64, a[1] as i64, a[2] as i64);
    let (r2, g2, b2) = (b[0] as i64, b[1] as i64, b[2] as i64);

    (((r2 - r1).pow(2) + (g2 - g1).pow(2) + (b2 - b1).pow(2)) as f64).sqrt() as i64
}

pub fn img_transparent(image: &DynamicImage) -> bool {
    let mut image = image.to_rgba8();
    for pixel in image.pixels_mut() {
        if pixel.0[3] != 255 {
            return true;
        }
    }

    false
}
pub fn pixel_transparent(pixel: &[u8; 4]) -> bool {
    pixel[3] < 100u8
}
