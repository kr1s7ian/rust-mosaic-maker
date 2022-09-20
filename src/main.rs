use average_color::{self, enums::Rgb, get_average_color};
use image::imageops::overlay;
use core::time;
use image::{imageops, DynamicImage, GenericImageView, ImageBuffer, Pixel, RgbImage, Progress, Rgba};
use serde::de::IntoDeserializer;
use std::collections::HashMap;
use std::{env, clone, result};
use std::ffi::OsStr;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::{fs, path::Path};
use tokio;

mod dithering;

fn is_png(path: &Path) -> bool {
    return path.extension().and_then(OsStr::to_str) == Some("png");
}

async fn load_blocks(path: &str) -> HashMap<[u8; 4], String> {
    let dir = fs::read_dir(path).expect(&format!("Error while reading files in {path}"));
    let mut result: HashMap<[u8; 4], String> = HashMap::new();

    for file in dir {
        if file.is_ok() {
            let file = file.unwrap();
            if is_png(&file.path()) {
                // this file is correcly read and is a png
                let string_path = file.path().to_string_lossy().to_string();
                println!("Reading {string_path}...");
                let avg_color = get_average_color(&string_path).await.unwrap().unwrap();
                let rgba = [avg_color.r, avg_color.g, avg_color.b, 255];
                result.entry(rgba).or_insert(string_path);
            }
        }
    }
  println!("Done reading entries of /{path}!");

    result
}

pub fn minecrafify_img(img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, blocks: HashMap<[u8; 4], String>)
-> DynamicImage {
    let (sx, sy) = img.dimensions();
    let mut output_img = DynamicImage::new_rgba8(sx * 16, sy * 16);
    for y in 0..sy {
        for x in 0..sx {
            let pixel = &img.get_pixel(x, y).0;
            let mut colors: Vec<[u8; 4]> = vec![];
            for item in blocks.clone() {
                colors.push(item.0);
            }

            let closest_color = dithering::rgb_closest(pixel, &colors);
            let block_path = blocks.get(&closest_color).unwrap();
            let mut block_img = image::open(block_path).unwrap();

            if pixel[3] == 0 {
                block_img = image::open("blocks/transparent.png").unwrap();
            }
            imageops::overlay(&mut output_img, &mut block_img, x as i64 * 16, y as i64 * 16);
        }
    }

    output_img
}

fn convert_threaded_png(path: &str, blocks: HashMap<[u8; 4], String>, threads: usize) {
    println!("Converting {path} to mosaic");
    let mut colors: Vec<[u8; 4]> = vec![];
    for item in blocks.clone() {
        colors.push(item.0);
    }

    let mut img = image::open(path).unwrap();
    let (sx, sy) = img.dimensions();

    let offset = sy / threads as u32;
    let mut start = 0u32;
    let mut end = offset+1;
    let output_img = DynamicImage::new_rgba8(sx * 16, sy * 16);
    let output_buffer = Arc::new(Mutex::new(output_img));

    let mut handles = vec![];
    for i in 0..threads {
        //println!("Started thread n{i}");
        let output_buffer = Arc::clone(&output_buffer);
        let blocks = blocks.clone();
        let colors = colors.clone();
        let mut slice = img.crop(0, start, sx, end).clone().into_rgba8();

        let handle = thread::spawn(move || {
            dithering::dither_img(&mut slice, &colors);
            let mut slice = minecrafify_img(&mut slice, blocks);
            overlay(&mut *output_buffer.lock().unwrap(), &mut slice, 0, (offset * i as u32) as i64 * 16 as i64);
            //slice.save(&format!("input/out{i}.png")).unwrap();
        });
        start = end;
        end += offset;
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
    //let result = minecrafify_img(&mut *output_buffer.lock().unwrap(), blocks);
    output_buffer.lock().unwrap().save("input/out.png").unwrap();
}

//  fn convert_dir(input_path: &str, blocks: HashMap<[u8; 3], String>) {
//      let colors: Vec<[u8; 3]> = blocks.clone().into_iter().map(|f| f.0).collect();
//      let dir = fs::read_dir(input_path).unwrap();
//      for item in dir {
//          let item = item.unwrap();
//          if is_png(&item.path()) {
//              //image processing
//              let input_img = image::open(item.path().to_string_lossy().to_string()).unwrap();
//              // carefull on this guy, he wants either rgba or rgb pngs not both
//              let mut buffer = input_img.to_rgba8();
//              let default_color = "blocks/transparent.png".to_string();

//              let (width, height) = buffer.dimensions();
//              let output_img = DynamicImage::new_rgba8(16 * width, 16 * height)
//                  .save("temp.png")
//                  .unwrap();
//              let mut output_img = image::open("temp.png").unwrap();
//              println!("converintg {}", item.path().to_string_lossy().to_string());

//              let mut dither = true;

//              for y in 0..height {
//                  for x in 0..width {
//                      let old_color = buffer.get_pixel(x, y).0;
//                      let new_color = closest(
//                          old_color[0].into(),
//                          old_color[1].into(),
//                          old_color[2].into(),
//                          colors,
//                      );
//                      let new_color = [new_color[0], new_color[1], new_color[2], old_color[3]];
//                      buffer.get_pixel_mut(x, y).0 = new_color;

//                      if dither && is_safe_index(x, y, width - 1, height - 1) {
//                          let err_r: f32 = f32::from(old_color[0]) - f32::from(new_color[0]);
//                          let err_g: f32 = f32::from(old_color[1]) - f32::from(new_color[1]);
//                          let err_b: f32 = f32::from(old_color[2]) - f32::from(new_color[2]);
//                          //println!("{},{},{}", err_r, err_g, err_b);
//                          //println!("{:?},{:?}", old_color, new_color);

//                          let r = buffer.get_pixel(x + 1, y).0[0];
//                          let g = buffer.get_pixel(x + 1, y).0[1];
//                          let b = buffer.get_pixel(x + 1, y).0[2];

//                          buffer.get_pixel_mut(x + 1, y).0[0] =
//                              (r as f64 + err_r as f64 * 7f64 / 16f64) as u8;
//                          buffer.get_pixel_mut(x + 1, y).0[1] =
//                              (g as f64 + err_g as f64 * 7f64 / 16f64) as u8;
//                          buffer.get_pixel_mut(x + 1, y).0[2] =
//                              (b as f64 + err_b as f64 * 7f64 / 16f64) as u8;

//                          let r = buffer.get_pixel(x - 1, y + 1).0[0];
//                          let g = buffer.get_pixel(x - 1, y + 1).0[1];
//                          let b = buffer.get_pixel(x - 1, y + 1).0[2];

//                          buffer.get_pixel_mut(x - 1, y + 1).0[0] =
//                              (r as f64 + err_r as f64 * 3f64 / 16f64) as u8;
//                          buffer.get_pixel_mut(x - 1, y + 1).0[1] =
//                              (g as f64 + err_g as f64 * 3f64 / 16f64) as u8;
//                          buffer.get_pixel_mut(x - 1, y + 1).0[2] =
//                              (b as f64 + err_b as f64 * 3f64 / 16f64) as u8;

//                          let r = buffer.get_pixel(x, y + 1).0[0];
//                          let g = buffer.get_pixel(x, y + 1).0[1];
//                          let b = buffer.get_pixel(x, y + 1).0[2];

//                          buffer.get_pixel_mut(x, y + 1).0[0] =
//                              (r as f64 + err_r as f64 * 5f64 / 16f64) as u8;
//                          buffer.get_pixel_mut(x, y + 1).0[1] =
//                              (g as f64 + err_g as f64 * 5f64 / 16f64) as u8;
//                          buffer.get_pixel_mut(x, y + 1).0[2] =
//                              (b as f64 + err_b as f64 * 5f64 / 16f64) as u8;

//                          let r = buffer.get_pixel(x + 1, y + 1).0[0];
//                          let g = buffer.get_pixel(x + 1, y + 1).0[1];
//                          let b = buffer.get_pixel(x + 1, y + 1).0[2];

//                          buffer.get_pixel_mut(x + 1, y + 1).0[0] =
//                              (r as f64 + err_r as f64 * 1f64 / 16f64) as u8;
//                          buffer.get_pixel_mut(x + 1, y + 1).0[01] =
//                              (g as f64 + err_g as f64 * 1f64 / 16f64) as u8;
//                          buffer.get_pixel_mut(x + 1, y + 1).0[2] =
//                              (b as f64 + err_b as f64 * 1f64 / 16f64) as u8;
//                      }
//                      let new_color = buffer.get_pixel(x, y).0;

//                      //// MINECRAFTIFY

//                      let img_path = blocks.get(&new_color[..3]).unwrap_or(&default_color);
//                      let mut img = image::open(img_path).unwrap();
//                      if !is_transparent(new_color) {
//                          img = image::open(&default_color).unwrap();
//                      }
//                      imageops::overlay(
//                          &mut output_img,
//                          &mut img,
//                          (x * 16u32).into(),
//                          (y * 16u32).into(),
//                      )
//                  }
//              }

//              fs::remove_file(item.path()).unwrap();
//              output_img.save(item.path()).unwrap();
//              drop(buffer);
//              drop(input_img);
//          } else {
//              if item.metadata().unwrap().is_dir() {
//                  let path = item.path().to_string_lossy().to_string();
//                  println!("converting direcotory: {}", &path);
//                  convert_dir(&path, blocks)
//              }
//          }
//      }
//  }
#[tokio::main]
async fn main() {
    let blocks = load_blocks("blocks/").await;
    //println!("{:?}", &blocks);

    println!("Insert number of threads you wish to use [MUST BE A INT]");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let threads = input
        .trim()
        .to_string()
        .parse::<usize>()
        .expect("Not a number!");

    let now = std::time::SystemTime::now();
    convert_threaded_png("input/img.png", blocks, threads);

    let elapsed = now.elapsed().unwrap();
    println!(
        "Conversion took {} ms using {} threads",
        elapsed.as_millis(),
        threads
    );
}
