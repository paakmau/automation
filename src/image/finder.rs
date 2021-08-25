use std::path::Path;

use crate::Result;

use super::{FlattenArray, GrayImage, Pattern, Screenshot};

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    RightDown,
    RightUp,
    LeftDown,
    LeftUp,
}

#[derive(Clone)]
struct StepRange(i32, i32, i32); // Start, end, and step

impl StepRange {
    fn into_rev(mut self) -> Self {
        self.0 -= 1;
        self.1 -= 1;

        // Swap
        let v = self.0;
        self.0 = self.1;
        self.1 = v;

        self.2 = -self.2;
        self
    }
}

impl Iterator for StepRange {
    type Item = i32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if (self.1 - self.0) * self.2 > 0 {
            let v = self.0;
            self.0 += self.2;
            Some(v)
        } else {
            None
        }
    }
}

struct LumaMatrix {
    width: u32,
    height: u32,
    square_sums: FlattenArray<u64>,
}

impl LumaMatrix {
    fn new(image: &GrayImage) -> LumaMatrix {
        let mut square_sums = FlattenArray::new(
            1 + image.width() as usize,
            1 + image.height() as usize,
            0u64,
        );

        let width = image.width();
        let height = image.height();
        for y in 0..height as usize {
            for x in 0..width as usize {
                let luma = image.pixel(x as u32, y as u32) as u64;
                square_sums[(y + 1, x + 1)] = square_sums[(y, x + 1)] - square_sums[(y, x)]
                    + square_sums[(y + 1, x)]
                    + luma * luma;
            }
        }

        LumaMatrix {
            width,
            height,
            square_sums,
        }
    }

    #[inline]
    fn square_sum_partial(&self, [y, x, yy, xx]: [u32; 4]) -> u64 {
        let (y, x, yy, xx) = (y as usize, x as usize, yy as usize, xx as usize);
        self.square_sums[(yy, xx)] - self.square_sums[(yy, x)] + self.square_sums[(y, x)]
            - self.square_sums[(y, xx)]
    }
}

pub struct Finder<'a> {
    screenshot: &'a Screenshot,
}

impl<'a> Finder<'a> {
    pub fn new(screenshot: &'a Screenshot) -> Self {
        Self { screenshot }
    }

    pub fn find(&self, pattern: &Pattern, dir: Direction) -> Option<(u32, u32)> {
        const THRESHOLD: f32 = 0.98;

        let image = GrayImage::from_screenshot(self.screenshot).into_compressed(pattern.factor());
        let packed_image = image.to_redundant_packed();
        let matrix = LumaMatrix::new(&image);

        let mut max_score = 0f32;
        let mut result = None;

        let mut y_range = StepRange(0, (matrix.height - pattern.height() + 1) as i32, 1);
        let mut x_range = StepRange(0, (matrix.width - pattern.width() + 1) as i32, 1);

        match dir {
            Direction::RightDown => {}
            Direction::RightUp => {
                y_range = y_range.into_rev();
            }
            Direction::LeftDown => {
                x_range = x_range.into_rev();
            }
            Direction::LeftUp => {
                y_range = y_range.into_rev();
                x_range = x_range.into_rev();
            }
        };

        for y in y_range {
            for x in x_range.clone() {
                let (y, x) = (y as u32, x as u32);

                const PACK: usize = 8;

                let mut score = 0u32;
                for dy in 0..pattern.height() {
                    for dx in (0..pattern.width()).step_by(PACK) {
                        let image_values = packed_image.pixels(x + dx, y + dy);
                        let pattern_values = pattern.packed_pixels(dx, dy);

                        let products = *image_values * *pattern_values;

                        for v in products.to_array() {
                            score += v as u32;
                        }
                    }
                }

                let norm =
                    ((matrix.square_sum_partial([y, x, y + pattern.height(), x + pattern.width()])
                        * pattern.square_sum()) as f32)
                        .sqrt();

                let score = score as f32 / norm;

                if score >= THRESHOLD && score > max_score {
                    max_score = score;
                    result = Some((
                        (x + (pattern.width() >> 1)) * pattern.factor(),
                        (y + (pattern.height() >> 1)) * pattern.factor(),
                    ));
                }
            }
        }

        result
    }

    pub fn save<T>(&self, path: T) -> Result<()>
    where
        T: AsRef<Path>,
    {
        self.screenshot.save(path)
    }
}
