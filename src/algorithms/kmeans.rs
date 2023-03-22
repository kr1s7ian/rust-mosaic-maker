use crate::utils::AverageColor;
use kmeans_colors::{get_kmeans, Kmeans, Sort};
use palette::{rgb::Rgb, IntoColor, Lab, Pixel, Srgb};

pub struct KmeansAlgorithm {}

impl AverageColor for KmeansAlgorithm {
    fn average_color(image: &image::DynamicImage) -> Option<[u8; 3]> {
        let buffer = image.as_bytes();
        let lab: Vec<Lab> = Srgb::from_raw_slice(buffer)
            .iter()
            .map(|x| x.into_format().into_color())
            .collect();

        // Iterate over the runs, keep the best results
        let mut result = Kmeans::new();
        for i in 0..3 {
            let run_result = get_kmeans(8, 20, 5.0, false, &lab, 72342792347 + i as u64);
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
                Some(raw_color)
            }
        }
    }
}
