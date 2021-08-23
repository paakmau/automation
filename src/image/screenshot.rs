use std::path::Path;

use image::{GenericImageView, RgbaImage};

use crate::error::Result;

#[derive(PartialEq)]
pub struct Pixel<'a> {
    bgra: &'a [u8],
}

impl<'a> Pixel<'a> {
    pub fn new(bgra: &'a [u8]) -> Self {
        Self { bgra }
    }

    pub fn r(&self) -> u8 {
        self.bgra[2]
    }

    pub fn g(&self) -> u8 {
        self.bgra[1]
    }

    pub fn b(&self) -> u8 {
        self.bgra[0]
    }

    pub fn a(&self) -> u8 {
        self.bgra[3]
    }

    pub fn luma(&self) -> u8 {
        const BGR_LUMA_FACTOR: [u32; 3] = [722, 7152, 2126];
        const SCALE: u32 = 10000;
        let mut luma = 0u32;
        for i in 0..BGR_LUMA_FACTOR.len() {
            luma += self.bgra[i] as u32 * BGR_LUMA_FACTOR[i];
        }
        (luma / SCALE) as u8
    }
}

#[derive(Debug)]
pub struct Screenshot {
    width: u32,
    height: u32,
    bgra_buf: Vec<u8>,
}

impl Screenshot {
    pub fn from_bgra_buf(width: u32, height: u32, bgra_buf: Vec<u8>) -> Result<Self> {
        if bgra_buf.len() as u32 != width * height * 4 {
            return Err("Unknown error".to_string());
        }
        Ok(Screenshot {
            width,
            height,
            bgra_buf,
        })
    }

    pub fn from_file<T>(path: T) -> Result<Self>
    where
        T: AsRef<Path>,
    {
        match image::open(path) {
            Ok(dyn_img) => Ok(Screenshot::from_bgra_buf(
                dyn_img.width(),
                dyn_img.height(),
                dyn_img.into_rgba8().into_raw(),
            )
            .unwrap()),
            _ => Err("Unknown error".to_string()),
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pixel(&self, x: u32, y: u32) -> Pixel {
        let head = (y * self.width + x) * 4;
        let head = head as usize;
        Pixel::new(&self.bgra_buf[head..head + 4])
    }

    pub fn save<T>(&self, path: T) -> Result<()>
    where
        T: AsRef<Path>,
    {
        let mut buf = self.bgra_buf.clone();
        Self::swap_chanel_r_and_b(&mut buf);
        let img = RgbaImage::from_raw(self.width, self.height, buf).unwrap();
        match img.save(path) {
            Ok(()) => Ok(()),
            _ => Err("Unknown error".to_string()),
        }
    }

    fn swap_chanel_r_and_b(buf: &mut Vec<u8>) {
        for i in (0..buf.len()).step_by(4) {
            buf.swap(i, i + 2);
        }
    }
}
