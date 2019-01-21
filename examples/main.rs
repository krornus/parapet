use std::fs::File;
use std::thread::sleep;
use std::time::Duration;

use parapet::image;
use memmap::MmapOptions;

fn main() {

    let manager = parapet::Manager::new()
        .expect("failed to init manager");


    let path = "/home/spowell/programming/rust/parapet/data/forest_river.jpg";
    let file = File::open(path).expect("failed to open file");
    let mmap = unsafe { MmapOptions::new().map(&file).expect("failed to mmap") };
    let image = image::load_from_memory(&mmap)
        .expect("failed to open image");


    for disp in manager.displays().expect("failed to iter displays").next() {
        disp.set(&image, parapet::ImageMode::Max)
            .expect("failed to set image");
    }

}
