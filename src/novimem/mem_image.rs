use image::{GenericImage, GenericImageView, ImageBuffer, RgbImage};
use num_integer::Roots;

use super::NoviMem;

pub struct MemImage {
    prev_snapshot: Vec<u8>,
    prev_addr: u64,
}

impl MemImage {
    pub fn new() -> MemImage {
        MemImage {
            prev_snapshot: Vec::<u8>::new(),
            prev_addr: 0,
        }
    }
    pub fn print_img(&mut self, mem: &mut NoviMem, addr: u64, size: usize) {
        // Get the block of memory we care about
        if let Some(mem_block) = mem.getval(addr, size) {
            if self.prev_snapshot.len() != size || self.prev_addr != addr {
                self.prev_snapshot.clear();
            }
            let imgx = size.sqrt();
            let imgy = (size / imgx) + 1;

            let img = ImageBuffer::from_fn(imgx as u32, imgy as u32, |x, y| {
                let idx = (x + (y * imgx as u32)) as usize;
                let is_oversize = idx >= mem_block.len();
                let is_comparable = !is_oversize && !self.prev_snapshot.is_empty();
                let r =
                    if is_oversize || (is_comparable && self.prev_snapshot[idx] < mem_block[idx]) {
                        255
                    } else {
                        mem_block[idx]
                    };
                let g = if is_oversize { 0 } else { mem_block[idx] };
                let b = if is_oversize {
                    255
                } else if is_comparable && self.prev_snapshot[idx] > mem_block[idx] {
                    255
                } else {
                    mem_block[idx]
                };
                image::Rgb([r, g, b])
            });

            let (width, height) = img.dimensions();
            println!("Image of dims {} x {}", width, height);
            img.save("./image.bmp").unwrap();

            self.prev_snapshot = mem_block;
            self.prev_addr = addr;
        } else {
            println!(
                "Unable to get memblock in print_img() at addr {:X} of size {}",
                addr, size
            );
        }
    }
}
