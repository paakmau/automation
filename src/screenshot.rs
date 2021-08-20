use std::path::Path;

use image::{ImageFormat, Rgba, RgbaImage};

use crate::error::Result;

#[derive(PartialEq)]
pub struct Pixel<'a> {
    rgba: &'a Rgba<u8>,
}

impl<'a> Pixel<'a> {
    pub fn new(rgba: &'a Rgba<u8>) -> Self {
        Self { rgba }
    }

    pub fn r(&self) -> u8 {
        self.rgba.0[0]
    }

    pub fn g(&self) -> u8 {
        self.rgba.0[1]
    }

    pub fn b(&self) -> u8 {
        self.rgba.0[2]
    }

    pub fn luma(&self) -> u8 {
        const SRGB_LUMA: [u32; 3] = [2126, 7152, 722];
        let mut luma = 0u32;
        for i in 0..SRGB_LUMA.len() {
            luma += self.rgba.0[i] as u32 * SRGB_LUMA[i];
        }
        (luma / 10000u32) as u8
    }
}

pub struct Screenshot {
    image: RgbaImage,
}

impl Screenshot {
    pub fn from_raw(width: u32, height: u32, rgba_data: Vec<u8>) -> Result<Self> {
        match RgbaImage::from_raw(width, height, rgba_data) {
            Some(image) => Ok(Screenshot { image }),
            None => Err("Data buffer not big enough".to_string()),
        }
    }

    pub fn from_png_buf(buf: &[u8]) -> Result<Self> {
        match image::load_from_memory_with_format(buf, ImageFormat::Png) {
            Ok(dyn_img) => Ok(Screenshot {
                image: dyn_img.into_rgba8(),
            }),
            _ => Err("Unknown error".to_string()),
        }
    }

    pub fn from_file<T>(path: T) -> Result<Self>
    where
        T: AsRef<Path>,
    {
        match image::open(path) {
            Ok(dyn_img) => Ok(Screenshot {
                image: dyn_img.into_rgba8(),
            }),
            _ => Err("Unknown error".to_string()),
        }
    }

    pub fn save<T>(&self, path: T) -> Result<()>
    where
        T: AsRef<Path>,
    {
        match self.image.save(path) {
            Ok(()) => Ok(()),
            _ => Err("Unknown error".to_string()),
        }
    }

    pub fn width(&self) -> u32 {
        self.image.width()
    }

    pub fn height(&self) -> u32 {
        self.image.height()
    }

    pub fn pixel(&self, x: u32, y: u32) -> Pixel {
        Pixel::new(&self.image.get_pixel(x, y))
    }
}
