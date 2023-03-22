use crate::mosaic::MosaicMaker;

mod algorithms;
mod mosaic;
mod utils;

use algorithms::histogram::HistogramAlgorithm;
use algorithms::kmeans::KmeansAlgorithm;

fn main() {
    let mut mosaic_maker = MosaicMaker::new((16, 16));
    mosaic_maker
        .load_pieces::<KmeansAlgorithm>("block")
        .unwrap();

    mosaic_maker
        .compose("test.png", false)
        .unwrap()
        .save("output.png")
        .unwrap();
}
