use std::path::Path;

use image::GenericImageView;
use wide::u16x8;

use crate::Result;

use super::{FlattenArray, Screenshot};

pub(super) struct GrayImage {
    width: u32,
    height: u32,
    buf: FlattenArray<u8>,
}

impl GrayImage {
    pub fn from_screenshot(screenshot: &Screenshot) -> Self {
        let width = screenshot.width();
        let height = screenshot.height();
        let mut buf = FlattenArray::new(width as usize, height as usize, 0u8);
        for y in 0..height {
            for x in 0..width {
                buf[(y as usize, x as usize)] = screenshot.pixel(x, y).luma();
            }
        }
        Self { width, height, buf }
    }

    #[inline]
    pub fn from_file_buf(buf: &[u8]) -> Result<Self> {
        match image::load_from_memory(buf) {
            Ok(dyn_img) => Ok(Self::from_raw(
                dyn_img.width(),
                dyn_img.height(),
                dyn_img.into_luma8().into_raw(),
            )
            .unwrap()),
            Err(_) => Err("Unknown error".to_string()),
        }
    }

    #[inline]
    pub fn from_raw(width: u32, height: u32, buf: Vec<u8>) -> Result<Self> {
        if width * height != buf.len() as u32 {
            return Err("Unknown error".to_string());
        }

        // Compress the image

        Ok(Self {
            width,
            height,
            buf: FlattenArray::from_vec(width as usize, buf),
        })
    }

    #[inline]
    pub fn pixel(&self, x: u32, y: u32) -> u8 {
        self.buf[(y as usize, x as usize)]
    }

    #[inline]
    pub fn into_compressed(self, factor: Option<u32>) -> CompressedGrayImage {
        CompressedGrayImage::from_gray_image(&self, factor)
    }
}

pub(super) struct CompressedGrayImage {
    factor: u32,
    width: u32,
    height: u32,
    buf: FlattenArray<u8>,
}

impl CompressedGrayImage {
    #[inline]
    pub fn from_gray_image(image: &GrayImage, factor: Option<u32>) -> Self {
        let factor =
            factor.unwrap_or(((image.width * image.height / 250) as f32).sqrt().sqrt() as u32);
        let factor = factor.max(2);

        let width = image.width / factor;
        let height = image.height / factor;
        let mut buf = FlattenArray::new(width as usize, height as usize, 0u32);

        for y in 0..image.height / factor * factor {
            for x in 0..image.width / factor * factor {
                buf[((y / factor) as usize, (x / factor) as usize)] += image.pixel(x, y) as u32;
            }
        }

        let buf = buf
            .into_vec()
            .into_iter()
            .map(|v| (v / factor / factor) as u8)
            .collect();
        let buf = FlattenArray::from_vec(width as usize, buf);

        Self {
            factor,
            width,
            height,
            buf,
        }
    }

    #[inline]
    pub fn factor(&self) -> u32 {
        self.factor
    }
    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }
    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }

    #[inline]
    pub fn pixel(&self, x: u32, y: u32) -> u8 {
        self.buf[(y as usize, x as usize)]
    }

    #[inline]
    pub fn to_redundant_packed(&self) -> RedundantPackedGrayImage {
        RedundantPackedGrayImage::from_compressed_gray_image(self)
    }

    #[inline]
    pub fn to_packed(&self) -> PackedGrayImage {
        PackedGrayImage::from_compressed_gray_image(self)
    }

    #[inline]
    pub fn save<T>(&self, path: T) -> Result<()>
    where
        T: AsRef<Path>,
    {
        let img = image::GrayImage::from_raw(self.width, self.height, self.buf.to_vec()).unwrap();
        img.save(path).map_err(|_| "Unknown error".to_string())
    }
}

pub(super) struct RedundantPackedGrayImage {
    buf: FlattenArray<u16x8>,
}

impl RedundantPackedGrayImage {
    const PACK: usize = 8;

    #[inline]
    pub fn from_compressed_gray_image(image: &CompressedGrayImage) -> Self {
        let width = image.width();
        let height = image.height();

        let mut buf = FlattenArray::new(width as usize, height as usize, u16x8::from(0u16));
        for y in 0..height {
            for x in 0..width {
                let mut pixels = [0u16; Self::PACK];
                for i in 0..Self::PACK.min((width - x) as usize) {
                    pixels[i] = image.pixel(x + i as u32, y) as u16;
                }
                buf[(y as usize, x as usize)] = u16x8::from(pixels);
            }
        }

        Self { buf }
    }

    #[inline]
    pub fn pixels(&self, x: u32, y: u32) -> &u16x8 {
        &self.buf[(y as usize, x as usize)]
    }
}

pub(super) struct PackedGrayImage {
    buf: FlattenArray<u16x8>,
}

impl PackedGrayImage {
    const PACK: usize = 8;

    #[inline]
    pub fn from_compressed_gray_image(image: &CompressedGrayImage) -> Self {
        let width = image.width();
        let height = image.height();

        let mut packed_width = width as usize / Self::PACK;
        if width as usize % Self::PACK != 0 {
            packed_width += 1;
        }
        let mut buf = FlattenArray::new(packed_width, height as usize, u16x8::from(0u16));
        for y in 0..height {
            for x in (0..width).step_by(Self::PACK) {
                let mut pixels = [0u16; Self::PACK];
                for i in 0..Self::PACK.min((width - x) as usize) {
                    pixels[i] = image.pixel(x + i as u32, y) as u16;
                }
                buf[(y as usize, x as usize / Self::PACK)] = u16x8::from(pixels);
            }
        }

        Self { buf }
    }

    #[inline]
    pub fn pixels(&self, x: u32, y: u32) -> &u16x8 {
        &self.buf[(y as usize, x as usize / Self::PACK)]
    }
}
