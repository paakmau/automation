use std::path::Path;

use image::RgbaImage;

use crate::error::Result;

pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Pixel {
    pub fn new(rgba: [u8; 4]) -> Self {
        Self {
            r: rgba[0],
            g: rgba[1],
            b: rgba[2],
            a: rgba[3],
        }
    }
}

pub struct Screenshot(RgbaImage);

impl Screenshot {
    pub fn from_raw(width: u32, height: u32, rgba_data: Vec<u8>) -> Option<Self> {
        RgbaImage::from_raw(width, height, rgba_data).map(|rgba_image| Self(rgba_image))
    }

    pub fn save<T>(&self, path: T) -> Result<()>
    where
        T: AsRef<Path>,
    {
        match self.0.save(path) {
            Ok(()) => Ok(()),
            _ => Err("Unknown error".to_string()),
        }
    }

    pub fn pixel(&self, x: u32, y: u32) -> Pixel {
        Pixel::new(self.0.get_pixel(x, y).0)
    }
}
