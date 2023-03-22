use image::DynamicImage;

use crate::utils::closest_color;

fn calutate_error(old_pixel: &[u8; 3], new_pixel: &[u8; 3]) -> [f32; 3] {
    [
        f32::from(old_pixel[0]) - f32::from(new_pixel[0]),
        f32::from(old_pixel[1]) - f32::from(new_pixel[1]),
        f32::from(old_pixel[2]) - f32::from(new_pixel[2]),
    ]
}

fn apply_error(error: &[f32; 3], pixel: &[u8; 3], coeff: f32) -> [u8; 3] {
    [
        (pixel[0] as f32 + error[0] * coeff / 16f32) as u8,
        (pixel[1] as f32 + error[1] * coeff / 16f32) as u8,
        (pixel[2] as f32 + error[2] * coeff / 16f32) as u8,
    ]
}

fn is_safe_index(x: u32, y: u32, mx: u32, my: u32) -> bool {
    (x > 0 && x < mx) && (y > 0 && y < my)
}

pub fn dither_img(img: &DynamicImage, group: &[[u8; 3]]) -> DynamicImage {
    let mut img = img.to_rgb8();
    let (sx, sy) = img.dimensions();
    for y in 0..sy {
        for x in 0..sx {
            let old_pixel = img.get_pixel(x, y).0;
            let closest_pixel = closest_color(group, &old_pixel);
            let new_pixel = closest_pixel;
            img.get_pixel_mut(x, y).0 = closest_pixel;

            if is_safe_index(x, y, sx - 1, sy - 1) {
                let error = calutate_error(&old_pixel, &new_pixel);

                let mut pixel = &mut img.get_pixel_mut(x + 1, y);
                pixel.0 = apply_error(&error, &pixel.0, 7.0);

                let mut pixel = &mut img.get_pixel_mut(x - 1, y + 1);
                pixel.0 = apply_error(&error, &pixel.0, 3.0);

                let mut pixel = &mut img.get_pixel_mut(x, y + 1);
                pixel.0 = apply_error(&error, &pixel.0, 5.0);

                let mut pixel = &mut img.get_pixel_mut(x + 1, y + 1);
                pixel.0 = apply_error(&error, &pixel.0, 1.0);
            }
        }
    }

    img.into()
}
