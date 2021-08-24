use std::path::Path;

use crate::Result;

use super::{CompressedGrayImage, GrayImage};

pub struct Pattern {
    image: CompressedGrayImage,
    square_sum: u64,
}

impl Pattern {
    #[inline]
    pub fn from_file_buf(buf: &[u8]) -> Result<Self> {
        GrayImage::from_file_buf(buf).map(|image| {
            let image = image.into_compressed(None);
            let mut square_sum = 0u64;
            for y in 0..image.height() {
                for x in 0..image.width() {
                    square_sum += (image.pixel(x, y) as u64).pow(2);
                }
            }
            Self { image, square_sum }
        })
    }

    #[inline]
    pub fn factor(&self) -> u32 {
        self.image.factor()
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
    pub fn pixels(&self, x: u32, y: u32, len: u32) -> &[u8] {
        self.image.pixels(x, y, len)
    }

    pub fn save<T>(&self, path: T) -> Result<()>
    where
        T: AsRef<Path>,
    {
        self.image.save(path)
    }
}
