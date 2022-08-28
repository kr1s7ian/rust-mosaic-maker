mod block;
use average_color::{self, enums::Rgb, get_average_color};
use image::{imageops, ImageBuffer, RgbImage, GenericImageView};
use std::collections::HashMap;
use std::{fs, path::Path};
use tokio;

#[tokio::main]
async fn main() {

  //let vec = vec!(
  //    [100u8, 35u8,100u8],
  //    [167u8, 98u8,130u8],
  //    [100u8, 60u8,220u8],
  //);

  //let result = closest(100, 30, 100, vec);

  //println!("{:?}", result);
  //return;
  let dir = fs::read_dir(Path::new("blocks/"));

    let mut valid_blocks = HashMap::new();
    let mut palette: Vec<[u8; 3]> = vec![];

    for item in dir.unwrap() {
        let name: String = String::from(item.as_ref().unwrap().file_name().to_str().unwrap())
            .chars()
            .rev()
            .skip(4)
            .collect();
        let name: String = name.chars().rev().collect();
        let block = block::BlockKind::from_name(&name);
        //println!("{}", name);

        match block {
            Some(value) => {
                let block_png = item.unwrap().path().to_str().unwrap().to_string();
                let avg_color = get_average_color(&block_png).await.unwrap().unwrap();
                let rgb: [u8; 3] = [avg_color.r, avg_color.g, avg_color.b];
                //valid_blocks.entry(rgb).or_insert(value);
                valid_blocks.entry(rgb).or_insert(block_png);
                palette.push(rgb);
                continue;
            }
            None => continue,
        }
    }


    type T = i64;

    pub fn distance(r1: T, g1: T , b1: T, r2: T, g2: T, b2: T) -> T {
        ((r2 - r1).pow(2) + (g2 - g1).pow(2) + (b2 - b1).pow(2)).pow(2)
    }

    pub fn closest(r1: T, g1: T , b1: T, vec: &Vec<[u8; 3]>) -> [u8; 3] {
        let mut biggest_diff: i64 = i64::max_value();
        let mut closest: [u8; 3] = [0u8,0u8,0u8];

        for (i, item) in vec.iter().enumerate() {
            let d = distance(r1, g1, b1, vec[i][0].into(), vec[i][1].into(), vec[i][2].into());
            if d < biggest_diff {
                biggest_diff = d;
                closest = vec[i];
            }
        }

        closest
    }
    //println!("{:?}", valid_blocks);
    for item in &valid_blocks {
        println! {"{:?}", item};
    }

    let input_img = image::open("input.png").unwrap();

    // carefull on this guy, he wants either rgba or rgb pngs not both
    let buffer = input_img.as_rgba8().unwrap();

    let default_color = "blocks/white_glazed_terracotta.png".to_string();


    let (width, height) = buffer.dimensions();
    let output_img = RgbImage::new(16 * width, 16 * height)
        .save("output.png")
        .unwrap();
    let mut output_img = image::open("output.png").unwrap();
    for x in 0..width {
        for y in 0..height {
            let color = buffer[(x, y)].0;
            let color = closest(color[0].into(), color[1].into(), color[2].into(), &palette);
            let img_path = valid_blocks.get(&color).unwrap_or(&default_color);
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
