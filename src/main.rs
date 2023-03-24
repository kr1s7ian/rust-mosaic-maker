use std::{
    error::Error,
    fmt::{format, Display},
    fs,
    path::Path,
};

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
    Recursive,
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
            Self::Recursive => {
                write!(
                    f,
                    "Error while recursively converting folder to mosaic, check if the output path already exists."
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

    if cli.recursive {
        fs::create_dir(&cli.output_path).map_err(|_| CliErrors::Recursive)?;
        compose_folder_recursively(&cli, &cli.input_path, &cli.output_path, &mosaic_maker)
            .map_err(|_| CliErrors::Recursive)?;
        return Ok(());
    }

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

pub fn compose_folder_recursively(
    cli: &Cli,
    input: &str,
    output: &str,
    mosaic_maker: &MosaicMaker,
) -> Result<(), Box<dyn Error>> {
    let dir = fs::read_dir(&input)?;
    for file in dir {
        let file = match file {
            Err(_) => continue,
            Ok(file) => file,
        };
        let filepath = file.path().to_string_lossy().to_string();
        let filename = file.file_name().to_string_lossy().to_string();
        let output_dir = format!("{}/{}", output, filename);
        println!("{}", &output_dir);

        if file.file_type()?.is_dir() {
            fs::create_dir(&output_dir)?;
            compose_folder_recursively(cli, &filepath, &output_dir, mosaic_maker)?;
        }

        if !is_png(&file.path()) {
            continue;
        }

        println!("{filename}");
        //let output = format!("{}/{}", output_dir, &filename);
        //println!("{output}");
        mosaic_maker
            .compose(&filepath, cli.dither)?
            .save(output_dir)?;
    }
    Ok(())
}
