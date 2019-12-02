use image::{GenericImage, GenericImageView, ImageBuffer, RgbImage};

use super::NoviMem;

pub struct MemImage {
    prev_snapshot: Vec<u8>,
}

impl MemImage {
    pub fn new() -> MemImage {
        MemImage {
            prev_snapshot: Vec::<u8>::new(),
        }
    }
    pub fn print_img(&mut self, mem: &mut NoviMem) {
        let offset = 0x3E48767;
        // if let Some(contents) = mem.get_region_contents("[heap]") {
        //     if !self.prev_snapshot.is_empty() {
        //         // let sz = contents.len() as u32;
        //         let sz = 10000;
        //         println!("Content length of {}: {} bytes", "[heap]", contents.len());
        //         let imgx = 100u32;
        //         let imgy = (sz / imgx) + 1;
        //         let img = ImageBuffer::from_fn(imgx, imgy, |x, y| {
        //             let idx = (x + (y * imgx) + offset) as usize;
        //             if idx < contents.len() {
        //                 let curr_byte = contents[idx];
        //                 let prev_byte = self.prev_snapshot[idx];
        //                 let r = contents[idx];
        //                 let g = contents[idx];
        //                 let b = if curr_byte != prev_byte { 255 } else { 0 };
        //                 image::Rgb([0, 0, b])
        //             } else {
        //                 image::Rgb([0xFF, 0x0, 0x0])
        //             }
        //         });

        //         let (width, height) = img.dimensions();
        //         println!("Image of dims {} x {}", width, height);
        //         img.save("./image.bmp").unwrap();
        //     }
        //     self.prev_snapshot = contents;
        // }
    }
}
