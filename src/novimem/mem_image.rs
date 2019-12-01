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
        if let Some(contents) = mem.get_region_contents("[heap]") {
            if !self.prev_snapshot.is_empty() {
                let sz = contents.len() as u32;
                // let sz = 10000;
                println!("Content length of {}: {} bytes", "[heap]", contents.len());
                let imgx = 100u32;
                let imgy = sz / imgx;
                let img = ImageBuffer::from_fn(imgx, imgy, |x, y| {
                    let idx = ((x +( y * imgx)) + offset - sz) as usize;
                    // image::Luma([contents[idx as usize]])
                    // let curr_byte = contents[idx];
                    // let prev_byte = self.prev_snapshot[idx];
                    // let r = if curr_byte != prev_byte { 255 } else { 0 };
                    let r = contents[idx];
                    let g = contents[idx];
                    let b = contents[idx];
                    // image::Rgb([r, 0, 0])
                    image::Luma([r])
                });

                let (width, height) = img.dimensions();
                println!("Image of dims {} x {}", width, height);
                img.save("./image.bmp").unwrap();
            }
            self.prev_snapshot = contents;
        }
    }
}
