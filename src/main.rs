use std::{fs, path::Path};

use crate::mosaic::MosaicMaker;

mod algorithms;
mod mosaic;
mod utils;

use algorithms::histogram::HistogramAlgorithm;
use utils::is_png;
//use algorithms::kmeans::KmeansAlgorithm;

fn main() {
    let mut mosaic_maker = MosaicMaker::new((16, 16));
    mosaic_maker
        //.load_pieces::<KmeansAlgorithm>("block")
        //.unwrap()
        .load_pieces::<HistogramAlgorithm>("block", false)
        .unwrap();

    mosaic_maker
        .compose("test.png", false)
        .unwrap()
        .save("output.png")
        .unwrap();

    //let path = Path::new("testfolder");
    //compose_folder(path, &mosaic_maker);
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
