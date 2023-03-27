#![allow(dead_code)]
use std::{error::Error, fs, path::Path};

use image::{imageops, DynamicImage};

use crate::{
    algorithms::dithering::dither_img,
    utils::{img_transparent, pixel_transparent, rgb_distance, AverageColor},
};
#[derive(Debug, Clone)]
pub struct Piece {
    src: String,
    average_color: [u8; 3],
}

#[derive(Debug, Clone)]
pub struct MosaicMaker {
    pieces: Vec<Piece>,
    piece_size: u32,
}

impl MosaicMaker {
    fn closest_piece_to_color<'a>(&'a self, target: &[u8; 3]) -> &'a Piece {
        let mut biggest_difference: i64 = i64::max_value();
        let mut closest = self.pieces.first().expect("Load at least one png!");

        for piece in self.pieces.iter() {
            let distance = rgb_distance(target, &piece.average_color);
            if distance < biggest_difference {
                biggest_difference = distance;
                closest = piece;
            }
        }

        closest
    }
}

impl MosaicMaker {
    pub fn new(piece_size: u32) -> Self {
        Self {
            pieces: vec![],
            piece_size,
        }
    }

    pub fn load_pieces(
        &mut self,
        path: &str,
        allow_transparency: bool,
        algorithm: Box<dyn AverageColor>,
    ) -> Result<&mut Self, Box<dyn Error>> {
        let pieces_path = Path::new(path);
        let folder = fs::read_dir(pieces_path)?;

        for file in folder {
            let file = file?;
            let path_string = file.path().to_string_lossy().to_string();

            let piece_img_path = path_string.as_str();
            let img = match image::open(piece_img_path) {
                Err(_) => {
                    println!("Ignoring: {path_string}, this file is not an image or is corrupted.");
                    continue;
                }
                Ok(img) => img,
            };

            if !allow_transparency && img_transparent(&img) {
                println!("Ignoring: {path_string}, this file contains transparent pixels.");
                continue;
            }

            println!("Loading: {path_string}...");
            let img = img.to_rgba8();

            let average_color = algorithm.average_color(&img.into());
            let average_color = match average_color {
                None => continue,
                Some(color) => color,
            };

            self.pieces.push(Piece {
                src: piece_img_path.to_string(),
                average_color,
            })
        }

        Ok(self)
    }

    pub fn clear_pieces(&mut self) {
        self.pieces.clear();
    }

    pub fn pieces_size(&self) -> u32 {
        self.piece_size
    }

    pub fn set_piece_size(&mut self, piece_size: u32) {
        self.piece_size = piece_size;
    }

    pub fn compose(
        &self,
        image: &DynamicImage,
        dithering: bool,
    ) -> Result<DynamicImage, Box<dyn Error>> {
        let mut target_image = image.to_rgba8();

        let (target_width, target_height) = target_image.dimensions();
        let (piece_width, piece_height) = (self.piece_size, self.piece_size);

        let mut output_img =
            DynamicImage::new_rgba8(target_width * piece_width, target_height * piece_height);

        if dithering {
            let available_colors: Vec<[u8; 3]> =
                self.pieces.iter().map(|p| p.average_color).collect();
            target_image = dither_img(&target_image.into(), &available_colors).to_rgba8();
        }

        for x in 0..target_width {
            for y in 0..target_height {
                let pixel = target_image.get_pixel(x, y);
                if pixel_transparent(&pixel.0) {
                    continue;
                }
                let rgb = [pixel.0[0], pixel[1], pixel[2]];
                let closest_piece = self.closest_piece_to_color(&rgb);

                let piece_img = image::open(&closest_piece.src)?;

                imageops::overlay(
                    &mut output_img,
                    &piece_img,
                    x as i64 * piece_width as i64,
                    y as i64 * piece_height as i64,
                );
            }
        }
        Ok(output_img)
    }
}
