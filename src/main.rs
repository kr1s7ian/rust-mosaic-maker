use std::{error::Error, fs, path::Path};

use crate::mosaic::MosaicMaker;

mod algorithms;
mod mosaic;
mod utils;

use algorithms::histogram::HistogramAlgorithm;
use clap::{Parser, Subcommand};
use utils::{is_png, AverageColor};
//use algorithms::kmeans::KmeansAlgorithm;

#[derive(Parser, Debug)]
pub struct Cli {
    input_path: String,
    output_path: String,
    pieces_folder: String,
    piece_size: u32,
    #[arg()]
    dither: Option<bool>,
    #[arg()]
    allow_transparent_pieces: Option<bool>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let allow_transparency = cli.allow_transparent_pieces.unwrap_or(true);
    let piece_size = (cli.piece_size, cli.piece_size);
    let pieces_path = cli.pieces_folder;
    let target_image = cli.input_path;
    let output_path = cli.output_path;
    let dithering = cli.dither.unwrap_or(false);

    let mut mosaic_maker = MosaicMaker::new(piece_size);
    mosaic_maker.load_pieces::<HistogramAlgorithm>(&pieces_path, allow_transparency)?;
    println!("Done loading pieces.");

    println!("Composing mosaic...");
    let output = mosaic_maker.compose(&target_image, dithering)?;
    println!("Done composing mosaic.");

    println!("Saving mosaic file...");
    output.save(&output_path)?;
    println!(
        "Succesfully generated mosaic from {pieces_path} folder and saved result to {output_path}."
    );

    Ok(())
}

pub fn compose_folder_recursively(path: &Path, mosaic_maker: &MosaicMaker) {
    let folder = fs::read_dir(path).unwrap();
    for file in folder {
        let file = match file {
            Err(_) => continue,
            Ok(file) => file,
        };
        if file.file_type().unwrap().is_dir() {
            compose_folder_recursively(&file.path(), &mosaic_maker)
        }
        if !is_png(&file.path()) {
            continue;
        }
        let path = file.path().to_string_lossy().to_string();
        let filename = file.file_name().to_string_lossy().to_string();
        println!("{filename}");
        let result = mosaic_maker
            .compose(&path, false)
            .unwrap()
            .save(path)
            .unwrap();
        //let result = make_black_transparent(&result)
        //   .to_rgba8()
        //  .save(path)
        // .unwrap();
    }
}
