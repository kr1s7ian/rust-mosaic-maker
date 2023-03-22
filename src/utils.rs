use image::DynamicImage;
use kmeans_colors::{get_kmeans, Calculate, Kmeans, MapColor, Sort};
use palette::{rgb::Rgb, FromColor, IntoColor, Lab, Pixel, Srgb};
use std::{ffi::OsStr, path::Path};

pub fn is_png(path: &Path) -> bool {
    path.extension().and_then(OsStr::to_str) == Some("png")
}

pub fn average_color(buffer: &[u8]) -> Option<[u8; 3]> {
    let lab: Vec<Lab> = Srgb::from_raw_slice(buffer)
        .iter()
        .map(|x| x.into_format().into_color())
        .collect();

    // Iterate over the runs, keep the best results
    let mut result = Kmeans::new();
    for i in 0..25 {
        let run_result = get_kmeans(8, 25, 5.0, false, &lab, 72342792347 + i as u64);
        if run_result.score < result.score {
            result = run_result;
        }
    }

    // Using the results from the previous example, process the centroid data
    let res = Lab::sort_indexed_colors(&result.centroids, &result.indices);

    // We can find the dominant color directly
    let dominant_color = Lab::get_dominant_color(&res);

    match dominant_color {
        None => None,
        Some(lab) => {
            let rgb_color: Rgb = lab.into_color();
            let raw_color: [u8; 3] = rgb_color.into_format().into_raw();
            return Some(raw_color);
        }
    }
}

pub fn get_prominent_color(image: &DynamicImage) -> Option<[u8; 3]> {
    let mut color_counts: std::collections::HashMap<[u8; 3], u32> =
        std::collections::HashMap::new();

    // Convert the image to RGB and iterate over the pixels
    let rgb_image = image.to_rgb8();
    for pixel in rgb_image.pixels() {
        // Get the color of the pixel as an (R, G, B) tuple
        let color = pixel.0;

        // Increment the counter for this color
        *color_counts.entry(color).or_insert(0) += 1;
    }

    // Find the color with the highest frequency
    color_counts
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(color, _)| color)
}

pub fn closest_color(available_colors: &[[u8; 3]], target: &[u8; 3]) -> [u8; 3] {
    let mut biggest_difference: i64 = i64::max_value();
    let mut closest = [0u8, 0u8, 0u8];

    for (i, color) in available_colors.iter().enumerate() {
        let distance = rgb_distance(&target, color);
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