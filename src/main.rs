use std::{error::Error, fmt::Display, fs, path::Path};

use crate::mosaic::MosaicMaker;

mod algorithms;
mod mosaic;
mod utils;

use clap::Parser;
use utils::is_png;
//use algorithms::kmeans::KmeansAlgorithm;

#[derive(Parser, Debug)]
pub struct Cli {
    input_path: String,
    output_path: String,
    pieces_folder: String,
    piece_size: u32,
    #[arg(short = 'r', long = "recursive")]
    recursive: bool,
    #[arg(short = 'd', long = "dither")]
    dither: bool,
    #[arg(short = 't', long = "transparent_pieces")]
    allow_transparent_pieces: bool,
    #[arg(short = 'i', long = "kmeans_iterations")]
    #[arg(default_value_t = 100)]
    kmeans_iterations: usize,
    #[arg(short = 's', long = "kmeans_min_score")]
    #[arg(default_value_t = 0.001)]
    kmeans_min_score: f32,
}

#[derive(Debug)]
enum CliErrors {
    LoadingPieces,
    CompsingMosaic,
    ErrorSavingImage,
}

impl Display for CliErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CompsingMosaic => {
                write!(f, "Error composing the mosaic, check if the input image path is valid and it is an image format.")
            }
            Self::LoadingPieces => {
                write!(
                    f,
                    "Error loading mosaic pieces, check if the mosaic pieces folder path is valid."
                )
            }
            Self::ErrorSavingImage => {
                write!(
                    f,
                    "Error saving the ouput image, check if the output path is valid and is an image format."
                )
            }
        }
    }
}
impl Error for CliErrors {}

fn run() -> Result<(), CliErrors> {
    let cli = Cli::parse();
    let mut mosaic_maker = MosaicMaker::new(cli.piece_size);
    println!("Loading pieces...");
    mosaic_maker
        .load_pieces(
            &cli.pieces_folder,
            cli.allow_transparent_pieces,
            cli.kmeans_iterations,
            cli.kmeans_min_score,
        )
        .map_err(|_| CliErrors::LoadingPieces)?;
    println!("Done loading pieces.");

    println!("Composing mosaic...");
    let output = mosaic_maker
        .compose(&cli.input_path, cli.dither)
        .map_err(|_| CliErrors::CompsingMosaic)?;
    println!("Done composing mosaic.");

    println!("Saving mosaic file...");
    output
        .save(&cli.output_path)
        .map_err(|_| CliErrors::ErrorSavingImage)?;
    println!(
        "Succesfully generated mosaic from {} folder and saved result to {}.",
        &cli.pieces_folder, &cli.output_path
    );

    Ok(())
}

fn main() {
    match run() {
        Err(e) => println!("{}", e.to_string()),
        Ok(_) => println!(""),
    }
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
