#![allow(dead_code)]
use std::{error::Error, fs, path::Path};

use image::{imageops, DynamicImage};

use crate::{
    algorithms::dithering::dither_img,
    utils::{is_png, rgb_distance, AverageColor},
};
#[derive(Debug, Clone)]
pub struct Piece {
    src: String,
    average_color: [u8; 3],
}

#[derive(Debug, Clone)]
pub struct MosaicMaker {
    pieces: Vec<Piece>,
    piece_size: (usize, usize),
}

impl MosaicMaker {
    fn closest_piece_to_color<'a>(&'a self, target: &[u8; 3]) -> &'a Piece {
        let mut biggest_difference: i64 = i64::max_value();
        let mut closest = self.pieces.first().unwrap();

        for piece in self.pieces.iter() {
            let distance = rgb_distance(&target, &piece.average_color);
            if distance < biggest_difference {
                biggest_difference = distance;
                closest = piece;
            }
        }

        closest
    }
}

impl MosaicMaker {
    pub fn new(piece_size: (usize, usize)) -> Self {
        Self {
            pieces: vec![],
            piece_size,
        }
    }

    pub fn load_pieces<T: AverageColor>(
        &mut self,
        path: &str,
    ) -> Result<&mut Self, Box<dyn Error>> {
        let pieces_path = Path::new(path);
        let folder = fs::read_dir(pieces_path)?;

        for file in folder {
            let file = file?;
            let path_string = file.path().to_string_lossy().to_string();
            if !is_png(&file.path()) {
                println!("Ignoring: {path_string}, this file is not a png or is corrupted.");
                continue;
            };
            println!("Loading: {path_string}...");
            let piece_img_path = path_string.as_str();

            let img = image::open(piece_img_path)?.to_rgb16();
            let average_color = T::average_color(&img.into());
            let average_color = match average_color {
                Some(color) => color,
                None => continue,
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

    pub fn pieces_size(&self) -> (usize, usize) {
        self.piece_size
    }

    pub fn set_piece_size(&mut self, piece_size: (usize, usize)) {
        self.piece_size = piece_size;
    }

    pub fn compose(
        &self,
        image_path: &str,
        dithering: bool,
    ) -> Result<DynamicImage, Box<dyn Error>> {
        let mut target_image = image::open(image_path)?.to_rgb8();
        let (w, h) = target_image.dimensions();
        let (piece_w, piece_h) = self.piece_size;
        let mut output_img = DynamicImage::new_rgb8(w * piece_w as u32, h * piece_h as u32);

        let available_colors: Vec<[u8; 3]> = self.pieces.iter().map(|p| p.average_color).collect();
        if dithering {
            target_image = dither_img(&target_image.into(), &available_colors).to_rgb8();
        }

        for x in 0..w {
            for y in 0..h {
                let pixel = target_image.get_pixel(x, y);
                let closest_piece = self.closest_piece_to_color(&pixel.0);

                let mut piece_img = image::open(&closest_piece.src)?;

                imageops::overlay(
                    &mut output_img,
                    &mut piece_img,
                    x as i64 * piece_w as i64,
                    y as i64 * piece_h as i64,
                );
            }
        }
        Ok(output_img)
    }
}
