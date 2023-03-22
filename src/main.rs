use crate::mosaic::MosaicMaker;

mod dithering;
mod mosaic;
mod utils;

fn main() {
    let mut mosaic_maker = MosaicMaker::new((16, 16));
    mosaic_maker.load_pieces("block").unwrap();

    mosaic_maker
        .compose("test.png", false)
        .unwrap()
        .save("output.png")
        .unwrap();
}
