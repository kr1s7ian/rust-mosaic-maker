use std::{error::Error, fmt::Display, fs};

use crate::{
    algorithms::{histogram::HistogramAlgorithm, kmeans::KmeansAlgorithm},
    mosaic::MosaicMaker,
    utils::AverageColor,
};

mod algorithms;
mod mosaic;
mod utils;

use clap::{Parser, ValueEnum};
use utils::is_png;
//use algorithms::kmeans::KmeansAlgorithm;

#[derive(Debug, Clone, ValueEnum)]
pub enum CliAlgorithms {
    Kmeans,
    Histogram,
}
impl Display for CliAlgorithms {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

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
    #[arg(short = 't', long = "use_transparent_pieces")]
    allow_transparent_pieces: bool,
    #[arg(value_enum, short = 'a', long = "algorithm", default_value = "kmeans")]
    algorithm: CliAlgorithms,
    #[arg(short = 'i', long = "kmeans_iterations")]
    #[arg(default_value_t = 1000)]
    kmeans_iterations: usize,
    #[arg(short = 'c', long = "kmeans_clusters")]
    #[arg(default_value_t = 1)]
    kmeans_clusters: usize,
    #[arg(short = 's', long = "kmeans_min_score")]
    #[arg(default_value_t = 0.0)]
    kmeans_min_score: f32,
}

fn run() {
    let cli = Cli::parse();
    let mut mosaic_maker = MosaicMaker::new(cli.piece_size);
    println!("Loading pieces...");

    let algorithm: Box<dyn AverageColor> = match cli.algorithm {
        CliAlgorithms::Histogram => Box::new(HistogramAlgorithm::new()),
        CliAlgorithms::Kmeans => Box::new(KmeansAlgorithm::new(
            cli.kmeans_iterations,
            cli.kmeans_clusters,
            cli.kmeans_min_score,
        )),
    };

    mosaic_maker
        .load_pieces(&cli.pieces_folder, cli.allow_transparent_pieces, algorithm)
        .expect(&format!("Error while loading pieces. Make sure that the piece path specified '{}' exists and is a folder.", &cli.pieces_folder));
    println!("Done loading pieces using {} algorithm.", cli.algorithm);

    if cli.recursive {
        let error_msg = &format!("Error while composing folder to mosaic recursively, Make sure input path '{}' is a valid folder and that output path '{}' does not already exist.", cli.input_path, cli.output_path);
        fs::create_dir(&cli.output_path).expect(error_msg);
        compose_folder_recursively(&cli, &cli.input_path, &cli.output_path, &mosaic_maker)
            .expect(error_msg);

        println!(
            "Done converting folder recursively to mosaic, saved to {} folder.",
            &cli.output_path
        );
    } else {
        println!("Composing mosaic...");
        let output = mosaic_maker.compose(&cli.input_path, cli.dither).expect(&format!("Error while composing mosaic from input image '{}', Make sure the path is valid and is an image format.", cli.input_path));

        println!("Done composing mosaic.");

        println!("Saving mosaic file...");
        output.save(&cli.output_path).expect(&format!("Error while saving to output path '{}'. Make sure the path ends with an image format and is valid.", cli.output_path));
        println!(
            "Succesfully generated mosaic from {} folder and saved result to {}.",
            &cli.pieces_folder, &cli.output_path
        );
    }
}

fn main() {
    run();
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
            println!("Ignoring {filepath}, This file is not an image or is corrupted.");
            continue;
        }
        mosaic_maker
            .compose(&filepath, cli.dither)?
            .save(output_dir)?;
    }
    Ok(())
}
