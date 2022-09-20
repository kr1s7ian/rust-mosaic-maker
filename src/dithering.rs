use image::{DynamicImage, ImageBuffer, Rgba};

pub fn calutate_error(old_pixel: &[u8; 4], new_pixel: &[u8; 4]) -> [f32; 3] {
    [
        f32::from(old_pixel[0]) - f32::from(new_pixel[0]),
        f32::from(old_pixel[1]) - f32::from(new_pixel[1]),
        f32::from(old_pixel[2]) - f32::from(new_pixel[2]),
    ]
}

pub fn apply_error(error: &[f32; 3], pixel: &[u8; 4], coeff: f32) -> [u8; 4] {
    [
        (pixel[0] as f32 + error[0] * coeff / 16f32) as u8,
        (pixel[1] as f32 + error[1] * coeff / 16f32) as u8,
        (pixel[2] as f32 + error[2] * coeff / 16f32) as u8,
        pixel[3],
    ]
}

pub fn rgb_dist(a: &[u8; 4], b: &[u8; 4]) -> i64 {
    let (r1, g1, b1) = (a[0] as i64, a[1] as i64, a[2] as i64);
    let (r2, g2, b2) = (b[0] as i64, b[1] as i64, b[2] as i64);

    (((r2 - r1).pow(2) + (g2 - g1).pow(2) + (b2 - b1).pow(2)) as f64).sqrt() as i64
}

pub fn rgb_closest(color: &[u8; 4], colors: &Vec<[u8; 4]>) -> [u8; 4] {
    let mut biggest_diff: i64 = i64::max_value();
    let mut closest: [u8; 4] = [0u8, 0u8, 0u8, 0u8];

    for (i, item) in colors.iter().enumerate() {
        let d = rgb_dist(color, &colors[i]);
        if d < biggest_diff {
            biggest_diff = d;
            closest = colors[i];
        }
    }

    closest
}

fn is_safe_index(x: u32, y: u32, mx: u32, my: u32) -> bool {
    (x > 0 && x < mx) && (y > 0 && y < my)
}

pub fn dither_img(img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, colors: &Vec<[u8; 4]>) {
    let (sx, sy) = img.dimensions();
    for y in 0..sy {
        for x in 0..sx {
            let old_pixel = img.get_pixel(x, y).0;
            let closest_pixel = rgb_closest(&old_pixel, colors);
            let new_pixel = [
                closest_pixel[0],
                closest_pixel[1],
                closest_pixel[2],
                old_pixel[3],
            ];
            img.get_pixel_mut(x, y).0 = new_pixel;

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
}
