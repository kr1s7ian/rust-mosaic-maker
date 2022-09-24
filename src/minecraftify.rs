use crate::dithering;
use average_color::{self, enums::Rgb, get_average_color};
use core::time;
use std::fmt::format;
use image::imageops::overlay;
use image::{
    imageops, DynamicImage, GenericImageView, ImageBuffer, Pixel, Progress, RgbImage, Rgba, ImageError,
};
use serde::de::IntoDeserializer;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::{clone, env, result};
use std::{fs, path::Path};

fn is_png(path: &Path) -> bool {
    return path.extension().and_then(OsStr::to_str) == Some("png");
}

#[derive(Clone)]
pub struct Blocks {
    map: HashMap<[u8; 4], String>,
    palette: Vec<[u8; 4]>,
}

impl Blocks {
    pub async fn load(path: &str) -> Result<Self, std::io::Error> {
        let dir = fs::read_dir(path)?;
        let mut map: HashMap<[u8; 4], String> = HashMap::new();
        let mut palette = vec![];

        for file in dir {
            if file.is_ok() {
                let file = file.unwrap();
                if is_png(&file.path()) {
                    // this file is correcly read and is a png
                    let string_path = file.path().to_string_lossy().to_string();
                    println!("Reading {string_path}...");
                    let avg_color = get_average_color(&string_path).await.unwrap().unwrap();
                    let rgba = [avg_color.r, avg_color.g, avg_color.b, 255];
                    map.entry(rgba).or_insert(string_path);
                    palette.push(rgba);
                }
            }
        }

        Ok(Self { map, palette })
    }

    pub fn map(&self) -> &HashMap<[u8; 4], String> {
        return &self.map;
    }

    pub fn palette(&self) -> &Vec<[u8; 4]>{
        return &self.palette
    }
}

pub fn minecrafify_img(
    img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    blocks: &Blocks,
) -> DynamicImage {
    let (sx, sy) = img.dimensions();
    let mut output_img = DynamicImage::new_rgba8(sx * 16, sy * 16);
    for y in 0..sy {
        for x in 0..sx {
            let pixel = &img.get_pixel(x, y).0;
            let closest_color = dithering::rgb_closest(pixel, &blocks.palette);
            let block_path = blocks.map().get(&closest_color).unwrap();
            let mut block_img = image::open(block_path).unwrap();

            if pixel[3] == 0 {
                block_img = image::open("blocks/transparent.png").unwrap();
            }
            imageops::overlay(
                &mut output_img,
                &mut block_img,
                x as i64 * 16,
                y as i64 * 16,
            );
        }
    }

    output_img
}

pub fn convert_threaded_png(path: &str, out: &str, blocks: Blocks, threads: usize) -> Result<(), ImageError> {
    println!("Converting {path} to mosaic");
    let mut img = image::open(path)?;
    let (sx, sy) = img.dimensions();

    let offset = sy / threads as u32;
    let mut start = 0u32;
    let mut end = offset + 1;
    let output_img = DynamicImage::new_rgba8(sx * 16, sy * 16);
    let output_buffer = Arc::new(Mutex::new(output_img));
    let blocks = Arc::new(blocks);

    let mut handles = vec![];
    for i in 0..threads {
        //println!("Started thread n{i}");
        let output_buffer = Arc::clone(&output_buffer);
        let blocks = Arc::clone(&blocks);
        let mut slice = img.crop(0, start, sx, end).clone().into_rgba8();

        let handle = thread::spawn(move || {
            dithering::dither_img(&mut slice, &blocks.palette());
            let mut slice = minecrafify_img(&mut slice, &blocks);
            overlay(
                &mut *output_buffer.lock().unwrap(),
                &mut slice,
                0,
                (offset * i as u32) as i64 * 16 as i64,
            );
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
    output_buffer.lock().unwrap().save(out).unwrap();

    Ok(())
}
