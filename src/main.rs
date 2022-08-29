use average_color::{self, enums::Rgb, get_average_color};
use image::{imageops, GenericImageView, ImageBuffer, RgbImage};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::{fs, path::Path};
use tokio;

fn is_png(path: &Path) -> bool {
    return path.extension().and_then(OsStr::to_str) == Some("png");
}

async fn load_blocks(path: &str) -> HashMap<[u8; 3], String> {
    let dir = fs::read_dir(path).expect(&format!("Error while reading files in {path}"));
    let mut result: HashMap<[u8; 3], String> = HashMap::new();

    for file in dir {
        if file.is_ok() {
            let file = file.unwrap();

            if is_png(&file.path()) {
                // this file is correcly read and is a png
                let string_path = file.path().to_string_lossy().to_string();
                let avg_color = get_average_color(&string_path).await.unwrap().unwrap();
                let rgb = [avg_color.r, avg_color.g, avg_color.b];
                result.entry(rgb).or_insert(string_path);
            }
        }
    }

    result
}

pub fn distance(r1: i64, g1: i64, b1: i64, r2: i64, g2: i64, b2: i64) -> i64 {
    (((r2 - r1).pow(2) + (g2 - g1).pow(2) + (b2 - b1).pow(2)) as f64 ).sqrt() as i64
}

pub fn closest(r1: i64, g1: i64, b1: i64, vec: &Vec<&[u8; 3]>) -> [u8; 3] {
    let mut biggest_diff: i64 = i64::max_value();
    let mut closest: [u8; 3] = [0u8, 0u8, 0u8];

    for (i, item) in vec.iter().enumerate() {
        let d = distance(
            r1,
            g1,
            b1,
            vec[i][0].into(),
            vec[i][1].into(),
            vec[i][2].into(),
        );
        if d < biggest_diff {
            biggest_diff = d;
            closest = *vec[i];
        }
    }

    closest
}

#[tokio::main]
async fn main() {
    let blocks = load_blocks("blocks/").await;
    let colors: Vec<&[u8; 3]> = blocks.iter().map(|f| f.0).collect();
    println!("{:?}", blocks);

    //image processing
    let input_img = image::open("input.png").unwrap();
    // carefull on this guy, he wants either rgba or rgb pngs not both
    let buffer = input_img.to_rgb8();
    let default_color = "blocks/white_glazed_terracotta.png".to_string();

    let (width, height) = buffer.dimensions();
    let output_img = RgbImage::new(16 * width, 16 * height)
        .save("output.png")
        .unwrap();
    let mut output_img = image::open("output.png").unwrap();

    for x in 0..width {
        for y in 0..height {
            let color = buffer[(x, y)].0;
            let color = closest(color[0].into(), color[1].into(), color[2].into(), &colors);
            let img_path = blocks.get(&color).unwrap_or(&default_color);
            let mut img = image::open(img_path).unwrap();
            imageops::overlay(
                &mut output_img,
                &mut img,
                (x * 16u32).into(),
                (y * 16u32).into(),
            );
            // convert rgb to block
            //buffer[(x,y)].0 = valid_blocks[0].1;
        }
    }

    output_img.save("output.png").unwrap();

    println!("Hello, world!");
}
