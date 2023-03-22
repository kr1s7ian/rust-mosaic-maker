use image::DynamicImage;

use crate::utils::AverageColor;

pub struct HistogramAlgorithm {}

impl AverageColor for HistogramAlgorithm {
    fn average_color(image: &DynamicImage) -> Option<[u8; 3]> {
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
}
