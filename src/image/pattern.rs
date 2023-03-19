use std::path::Path;

use std::simd::u16x8;

use crate::Result;

use super::{GrayImage, PackedGrayImage};

pub struct Pattern {
    factor: u32,
    image: GrayImage,
    packed_image: PackedGrayImage,
    square_sum: u64,
}

impl Pattern {
    #[inline]
    pub fn from_file_buf(buf: &[u8]) -> Result<Self> {
        GrayImage::from_file_buf(buf).map(|image| {
            let factor = ((image.width() * image.height() / 160) as f32)
                .sqrt()
                .sqrt() as u32;
            let factor = factor.max(2);

            let image = image.into_compressed(factor);
            let packed_image = image.to_packed();
            let mut square_sum = 0u64;
            for y in 0..image.height() {
                for x in 0..image.width() {
                    square_sum += (image.pixel(x, y) as u64).pow(2);
                }
            }

            Self {
                factor,
                image,
                packed_image,
                square_sum,
            }
        })
    }

    #[inline]
    pub fn factor(&self) -> u32 {
        self.factor
    }
    #[inline]
    pub fn width(&self) -> u32 {
        self.image.width()
    }
    #[inline]
    pub fn height(&self) -> u32 {
        self.image.height()
    }

    #[inline]
    pub fn square_sum(&self) -> u64 {
        self.square_sum
    }

    #[inline]
    pub fn pixel(&self, x: u32, y: u32) -> u8 {
        self.image.pixel(x, y)
    }

    #[inline]
    pub fn packed_pixels(&self, x: u32, y: u32) -> &u16x8 {
        self.packed_image.pixels(x, y)
    }

    pub fn save<T>(&self, path: T) -> Result<()>
    where
        T: AsRef<Path>,
    {
        self.image.save(path)
    }
}
