use std::{
    error::Error,
    fmt::Display,
    fs::{self, File},
    io::BufWriter,
    path::Path,
};

use crate::{
    algorithms::{histogram::HistogramAlgorithm, kmeans::KmeansAlgorithm},
    mosaic::MosaicMaker,
    utils::{is_gif, AverageColor},
};

mod algorithms;
mod mosaic;
mod utils;

use clap::{Parser, ValueEnum};
use image::{
    codecs::gif::{GifDecoder, GifEncoder},
    AnimationDecoder, DynamicImage, Frame,
};

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
    #[arg(default_value_t = 10)]
    kmeans_iterations: usize,
    #[arg(short = 'c', long = "kmeans_clusters")]
    #[arg(default_value_t = 1)]
    kmeans_clusters: usize,
    #[arg(short = 's', long = "kmeans_min_score")]
    #[arg(default_value_t = 0.0)]
    kmeans_min_score: f32,
}

fn run() -> Option<()> {
    let cli = Cli::parse();
    let mut mosaic_maker = MosaicMaker::new(cli.piece_size);

    let algorithm: Box<dyn AverageColor> = match cli.algorithm {
        CliAlgorithms::Histogram => Box::new(HistogramAlgorithm::new()),
        CliAlgorithms::Kmeans => Box::new(KmeansAlgorithm::new(
            cli.kmeans_iterations,
            cli.kmeans_clusters,
            cli.kmeans_min_score,
        )),
    };

    println!("Loading pieces...");
    mosaic_maker
        .load_pieces(&cli.pieces_folder, cli.allow_transparent_pieces, algorithm)
        .unwrap_or_else(|_| panic!("Error while loading pieces. Make sure that the piece path specified '{}' exists and is a folder.", &cli.pieces_folder));
    println!("Done loading pieces using {} algorithm.", cli.algorithm);

    if cli.recursive {
        let error_msg = format!("Error while composing folder to mosaic recursively, Make sure input path '{}' is a valid folder and that output path '{}' does not already exist.", cli.input_path, cli.output_path);
        fs::create_dir(&cli.output_path).expect(&error_msg);
        compose_folder_recursively(&cli, &cli.input_path, &cli.output_path, &mosaic_maker)
            .expect(&error_msg);

        println!(
            "Done converting folder recursively to mosaic, saved to {} folder.",
            &cli.output_path
        );

        return None;
    }

    let input_path = Path::new(&cli.input_path);
    if is_gif(input_path) {
        println!("Gifs might take a while to convert...");
        compose_gif(&cli, &mosaic_maker).unwrap_or_else(
            |_| panic!(
            "Error while converting gif to mosaic, make sure input_path '{}' and output_path '{}' are valid paths.",
            &cli.input_path, &cli.output_path),
        );

        return None;
    }
    let img = image::open(&cli.input_path).expect("Error while opening input_path image");
    println!("Composing '{}'...", &cli.input_path);
    let output = mosaic_maker.compose(&img, cli.dither).unwrap_or_else(|_| panic!("Error while composing mosaic from input image '{}', Make sure the path is valid and is an image format.", cli.input_path));

    println!("Saving '{}'...", &cli.output_path);
    output.save(&cli.output_path).unwrap_or_else(|_| panic!("Error while saving to output path '{}'. Make sure the path ends with an image format and is valid.", cli.output_path));

    Some(())
}

fn main() {
    run().map(|_| std::process::exit(0));
    println!("Done!");
}

pub fn compose_gif(cli: &Cli, mosaic_maker: &MosaicMaker) -> Result<(), Box<dyn Error>> {
    let file = File::open(&cli.input_path)?;
    let decoder = GifDecoder::new(file)?;

    let frames = decoder.into_frames();
    let frames = frames.collect_frames()?;
    let total_frames = frames.len();
    println!("Composing '{}'...", &cli.input_path);

    let mut current_frame = 0;
    let frames = frames
        .iter()
        .map(|f| {
            println!("Frame [{}/{}]", &current_frame, &total_frames);
            let top = f.top();
            let left = f.left();
            let delay = f.delay();

            let image: DynamicImage = f.clone().into_buffer().into();
            let mosaic = mosaic_maker
                .compose(&image, cli.dither)
                .unwrap_or_else(|_| {
                    panic!("Error while composing mosaic frame {}", &current_frame)
                });

            let frame = Frame::from_parts(mosaic.into_rgba8(), left, top, delay);
            current_frame += 1;
            frame
        })
        .collect::<Vec<Frame>>();
    println!("Saving '{}'...", &cli.output_path);

    let mut file = BufWriter::new(File::create(&cli.output_path)?);
    let mut encoder = GifEncoder::new(&mut file);
    encoder.set_repeat(image::codecs::gif::Repeat::Infinite)?;

    let mut current_frame = 0;
    for frame in frames {
        current_frame += 1;
        let progress = ((current_frame as f32 / total_frames as f32) * 100f32) as usize;
        println!("Saving {}%", progress);
        encoder.encode_frame(frame)?;
    }

    Ok(())
}

pub fn compose_folder_recursively(
    cli: &Cli,
    input: &str,
    output: &str,
    mosaic_maker: &MosaicMaker,
) -> Result<(), Box<dyn Error>> {
    let dir = fs::read_dir(input)?;
    for file in dir {
        let file = match file {
            Err(_) => continue,
            Ok(file) => file,
        };
        let filepath = file.path().to_string_lossy().to_string();
        let filename = file.file_name().to_string_lossy().to_string();
        let output_dir = format!("{}/{}", output, filename);
        println!("{}", &output_dir);

        let is_dir = file.file_type()?.is_dir();
        if is_dir {
            fs::create_dir(&output_dir)?;
            compose_folder_recursively(cli, &filepath, &output_dir, mosaic_maker)?;
        }

        let img = image::open(&filepath);
        let img = match img {
            Err(_) => {
                println!("Ignoring {filepath}, This file is not an image or is corrupted.");
                continue;
            }
            Ok(img) => img,
        };
        mosaic_maker.compose(&img, cli.dither)?.save(output_dir)?;
    }
    Ok(())
}
