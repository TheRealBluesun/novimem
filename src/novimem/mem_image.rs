use image::{GenericImage, GenericImageView, ImageBuffer, RgbImage};
use num_integer::Roots;

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
    pub fn print_img(&mut self, mem: &mut NoviMem, addr: u64, size: usize) {
        // Get the block of memory we care about
        if let Some(mem_block) = mem.getval(addr, size) {
            let imgx = size.sqrt();
            let imgy = (size / imgx) + 1;

            let img = ImageBuffer::from_fn(imgx as u32, imgy as u32, |x, y| {
                let idx = (x + (y * imgx as u32)) as usize;
                let r = if idx >= mem_block.len() {
                    0
                } else {
                    mem_block[idx]
                };
                let g = if idx >= mem_block.len() {
                    0
                } else {
                    mem_block[idx]
                };
                let b = if idx >= mem_block.len() {
                    0
                } else {
                    mem_block[idx]
                };
                image::Rgb([r, g, b])
            });

            let (width, height) = img.dimensions();
            println!("Image of dims {} x {}", width, height);
            img.save("./image.bmp").unwrap();

            self.prev_snapshot = mem_block;
        } else {
            println!(
                "Unable to get memblock in print_img() at addr {:X} of size {}",
                addr, size
            );
        }

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
