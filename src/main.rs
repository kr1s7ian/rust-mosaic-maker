use average_color::{self, enums::Rgb, get_average_color};
use image::{imageops, DynamicImage, GenericImageView, ImageBuffer, Pixel, RgbImage};
use serde::de::IntoDeserializer;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::sync::{Arc, Mutex};
use std::thread;
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
    (((r2 - r1).pow(2) + (g2 - g1).pow(2) + (b2 - b1).pow(2)) as f64).sqrt() as i64
}

pub fn closest(r1: i64, g1: i64, b1: i64, vec: &Vec<[u8; 3]>) -> [u8; 3] {
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
            closest = vec[i];
        }
    }

    closest
}

fn is_safe_index(x: u32, y: u32, mx: u32, my: u32) -> bool {
    (x > 0 && x < mx) && (y > 0 && y < my)
}

fn make_transparent(
    img: DynamicImage,
    colors: &Vec<&[u8; 3]>,
) -> ImageBuffer<image::Rgba<u8>, Vec<u8>> {
    let mut img = img.into_rgba8();
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        if pixel.0[0] <= 10 || pixel.0[0] >= 254 {
            let is_white: bool = (pixel.0[0] == 255 && pixel.0[1] == 255 && pixel.0[2] == 255);
            let is_black: bool = (pixel.0[0] == 0 && pixel.0[1] == 0 && pixel.0[2] == 0);

            if is_white || is_black {
                pixel.0 = [0u8, 0u8, 0u8, 0u8];
            }
        }
    }

    img
}

fn is_transparent(rgba: [u8; 4]) -> bool {
    return rgba[3] > 150u8;
}

//  fn convert_png(img: &mut DynamicImage, blocks: &HashMap<[u8; 3], String>) {
//      let img = img.as_mut_rgba8().unwrap();

//      let colors: Vec<&[u8; 3]> = blocks.iter().map(|f| f.0).collect();
//      let mut transparent_img = "blocks/transparent.png".to_string();
//      let dither = true;

//      let (sx, sy) = img.dimensions();
//              for y in 0..sy{
//                  for x in 0..sx {
//                      let old_color = img.get_pixel(x, y).0;
//                      let new_color = closest(
//                          old_color[0].into(),
//                          old_color[1].into(),
//                          old_color[2].into(),
//                          colors,
//                      );
//                      let new_color = [new_color[0], new_color[1], new_color[2], old_color[3]];
//                      img.get_pixel_mut(x, y).0 = new_color;

//                      if dither && is_safe_index(x, y, sx - 1, sy - 1) {
//                          let err_r: f32 = f32::from(old_color[0]) - f32::from(new_color[0]);
//                          let err_g: f32 = f32::from(old_color[1]) - f32::from(new_color[1]);
//                          let err_b: f32 = f32::from(old_color[2]) - f32::from(new_color[2]);
//                          //println!("{},{},{}", err_r, err_g, err_b);
//                          //println!("{:?},{:?}", old_color, new_color);

//                          let r = img.get_pixel(x + 1, y).0[0];
//                          let g = img.get_pixel(x + 1, y).0[1];
//                          let b = img.get_pixel(x + 1, y).0[2];

//                          img.get_pixel_mut(x + 1, y).0[0] =
//                              (r as f64 + err_r as f64 * 7f64 / 16f64) as u8;
//                          img.get_pixel_mut(x + 1, y).0[1] =
//                              (g as f64 + err_g as f64 * 7f64 / 16f64) as u8;
//                          img.get_pixel_mut(x + 1, y).0[2] =
//                              (b as f64 + err_b as f64 * 7f64 / 16f64) as u8;

//                          let r = img.get_pixel(x - 1, y + 1).0[0];
//                          let g = img.get_pixel(x - 1, y + 1).0[1];
//                          let b = img.get_pixel(x - 1, y + 1).0[2];

//                          img.get_pixel_mut(x - 1, y + 1).0[0] =
//                              (r as f64 + err_r as f64 * 3f64 / 16f64) as u8;
//                          img.get_pixel_mut(x - 1, y + 1).0[1] =
//                              (g as f64 + err_g as f64 * 3f64 / 16f64) as u8;
//                          img.get_pixel_mut(x - 1, y + 1).0[2] =
//                              (b as f64 + err_b as f64 * 3f64 / 16f64) as u8;

//                          let r = img.get_pixel(x, y + 1).0[0];
//                          let g = img.get_pixel(x, y + 1).0[1];
//                          let b = img.get_pixel(x, y + 1).0[2];

//                          img.get_pixel_mut(x, y + 1).0[0] =
//                              (r as f64 + err_r as f64 * 5f64 / 16f64) as u8;
//                          img.get_pixel_mut(x, y + 1).0[1] =
//                              (g as f64 + err_g as f64 * 5f64 / 16f64) as u8;
//                          img.get_pixel_mut(x, y + 1).0[2] =
//                              (b as f64 + err_b as f64 * 5f64 / 16f64) as u8;

//                          let r = img.get_pixel(x + 1, y + 1).0[0];
//                          let g = img.get_pixel(x + 1, y + 1).0[1];
//                          let b = img.get_pixel(x + 1, y + 1).0[2];

//                          img.get_pixel_mut(x + 1, y + 1).0[0] =
//                              (r as f64 + err_r as f64 * 1f64 / 16f64) as u8;
//                          img.get_pixel_mut(x + 1, y + 1).0[1] =
//                              (g as f64 + err_g as f64 * 1f64 / 16f64) as u8;
//                          img.get_pixel_mut(x + 1, y + 1).0[2] =
//                              (b as f64 + err_b as f64 * 1f64 / 16f64) as u8;
//                      }
//                      let new_color = img.get_pixel(x, y).0;
//  }
//              }
//  }

fn convert_threaded_png(path: &str, blocks: HashMap<[u8; 3], String>, threads: usize) {
    let colors: Vec<[u8; 3]> = blocks.clone().into_iter().map(|f| f.0).collect();

    let img = image::open(path).unwrap();
    let (sx, sy) = img.dimensions();

    let offset = sy / threads as u32;
    let mut start = 0u32;
    let mut end = offset;
    let output_img = DynamicImage::new_rgba8(sx * 16, sy * 16);
    let buffer = Arc::new(Mutex::new(img.to_rgba8()));
    let output_buffer = Arc::new(Mutex::new(output_img.to_rgba8()));

    let mut handles = vec![];
    let dither = true;
    for i in 0..threads {
        println!("thread n {i}");
        let buffer = Arc::clone(&buffer);
        let output_buffer = Arc::clone(&output_buffer);
        let blocks = blocks.clone();
        let colors = colors.clone();
        let handle = thread::spawn(move || {
            for y in start..end {
                for x in 0..sx {
                    let old_color = buffer.lock().unwrap().get_pixel(x, y).0;
                    let new_color = closest(
                        old_color[0].into(),
                        old_color[1].into(),
                        old_color[2].into(),
                        &colors,
                    );
                    let new_color = [new_color[0], new_color[1], new_color[2], old_color[3]];
                    buffer.lock().unwrap().get_pixel_mut(x, y).0 = new_color;

                    if dither && is_safe_index(x, y, sx - 1, sy - 1) {
                        let err_r: f32 = f32::from(old_color[0]) - f32::from(new_color[0]);
                        let err_g: f32 = f32::from(old_color[1]) - f32::from(new_color[1]);
                        let err_b: f32 = f32::from(old_color[2]) - f32::from(new_color[2]);
                        //println!("{},{},{}", err_r, err_g, err_b);
                        //println!("{:?},{:?}", old_color, new_color);

                        let r = buffer.lock().unwrap().get_pixel(x + 1, y).0[0];
                        let g = buffer.lock().unwrap().get_pixel(x + 1, y).0[1];
                        let b = buffer.lock().unwrap().get_pixel(x + 1, y).0[2];

                        buffer.lock().unwrap().get_pixel_mut(x + 1, y).0[0] =
                            (r as f64 + err_r as f64 * 7f64 / 16f64) as u8;
                        buffer.lock().unwrap().get_pixel_mut(x + 1, y).0[1] =
                            (g as f64 + err_g as f64 * 7f64 / 16f64) as u8;
                        buffer.lock().unwrap().get_pixel_mut(x + 1, y).0[2] =
                            (b as f64 + err_b as f64 * 7f64 / 16f64) as u8;

                        let r = buffer.lock().unwrap().get_pixel(x - 1, y + 1).0[0];
                        let g = buffer.lock().unwrap().get_pixel(x - 1, y + 1).0[1];
                        let b = buffer.lock().unwrap().get_pixel(x - 1, y + 1).0[2];

                        buffer.lock().unwrap().get_pixel_mut(x - 1, y + 1).0[0] =
                            (r as f64 + err_r as f64 * 3f64 / 16f64) as u8;
                        buffer.lock().unwrap().get_pixel_mut(x - 1, y + 1).0[1] =
                            (g as f64 + err_g as f64 * 3f64 / 16f64) as u8;
                        buffer.lock().unwrap().get_pixel_mut(x - 1, y + 1).0[2] =
                            (b as f64 + err_b as f64 * 3f64 / 16f64) as u8;

                        let r = buffer.lock().unwrap().get_pixel(x, y + 1).0[0];
                        let g = buffer.lock().unwrap().get_pixel(x, y + 1).0[1];
                        let b = buffer.lock().unwrap().get_pixel(x, y + 1).0[2];

                        buffer.lock().unwrap().get_pixel_mut(x, y + 1).0[0] =
                            (r as f64 + err_r as f64 * 5f64 / 16f64) as u8;
                        buffer.lock().unwrap().get_pixel_mut(x, y + 1).0[1] =
                            (g as f64 + err_g as f64 * 5f64 / 16f64) as u8;
                        buffer.lock().unwrap().get_pixel_mut(x, y + 1).0[2] =
                            (b as f64 + err_b as f64 * 5f64 / 16f64) as u8;

                        let r = buffer.lock().unwrap().get_pixel(x + 1, y + 1).0[0];
                        let g = buffer.lock().unwrap().get_pixel(x + 1, y + 1).0[1];
                        let b = buffer.lock().unwrap().get_pixel(x + 1, y + 1).0[2];

                        buffer.lock().unwrap().get_pixel_mut(x + 1, y + 1).0[0] =
                            (r as f64 + err_r as f64 * 1f64 / 16f64) as u8;
                        buffer.lock().unwrap().get_pixel_mut(x + 1, y + 1).0[01] =
                            (g as f64 + err_g as f64 * 1f64 / 16f64) as u8;
                        buffer.lock().unwrap().get_pixel_mut(x + 1, y + 1).0[2] =
                            (b as f64 + err_b as f64 * 1f64 / 16f64) as u8;
                    }
                    let new_color = buffer.lock().unwrap().get_pixel(x, y).0;

                    // MINECRAFTIFY
                    let img_path = blocks.get(&new_color[..3]).unwrap();
                    let mut img = image::open(img_path).unwrap();
                    if !is_transparent(new_color) {
                        img = image::open("blocks/transparent.png").unwrap();
                    }

                    imageops::overlay(
                        &mut *output_buffer.lock().unwrap(),
                        &mut img,
                        (x * 16u32).into(),
                        (y * 16u32).into(),
                    );
                }
            }
        });
        start = end;
        end += offset;
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }

    output_buffer
        .lock()
        .unwrap()
        .save("input/output.png")
        .unwrap();
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
    println!("{:?}", &blocks);

    convert_threaded_png("input/img.png", blocks, 16);

    //convert_dir("input", &blocks);
    //convert_dir("input/", &blocks);

    println!("Hello, world!");
}
