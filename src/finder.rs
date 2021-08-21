use crate::Screenshot;

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
    sums: Vec<Vec<u64>>,
    width: u32,
    height: u32,
}

impl LumaMatrix {
    fn new(screenshot: &Screenshot) -> LumaMatrix {
        let mut sums =
            vec![vec![0u64; 1 + screenshot.width() as usize]; 1 + screenshot.height() as usize];

        let luma = |y, x| screenshot.pixel(x, y).luma() as u64;

        let width = screenshot.width();
        let height = screenshot.height();
        for y in 0..height as usize {
            for x in 0..width as usize {
                sums[y + 1][x + 1] =
                    sums[y][x + 1] + sums[y + 1][x] - sums[y][x] + luma(y as u32, x as u32);
            }
        }

        LumaMatrix {
            sums,
            width,
            height,
        }
    }

    fn luma_sum(&self) -> u64 {
        self.sums[self.height as usize][self.width as usize]
    }

    fn partial_luma_sum(&self, [y, x, yy, xx]: [u32; 4]) -> u64 {
        let [y, x, yy, xx] = [y as usize, x as usize, yy as usize, xx as usize];
        self.sums[yy][xx] + self.sums[y][x] - self.sums[yy][x] - self.sums[y][xx]
    }
}

pub struct Finder<'a> {
    screenshot: &'a Screenshot,
    luma_matrix: LumaMatrix,
}

impl<'a> Finder<'a> {
    pub fn new(screenshot: &'a Screenshot) -> Self {
        Self {
            screenshot,
            luma_matrix: LumaMatrix::new(screenshot),
        }
    }

    pub fn find(&self, pattern: &Screenshot, dir: Direction) -> Option<(u32, u32)> {
        let pattern_hash = LumaMatrix::new(pattern).luma_sum();
        let screenshot = self.screenshot;

        let mut y_range = StepRange(0, (screenshot.height() - pattern.height() + 1) as i32, 1);
        let mut x_range = StepRange(0, (screenshot.width() - pattern.width() + 1) as i32, 1);
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
            'x: for x in x_range.clone() {
                let y = y as u32;
                let x = x as u32;
                // First check luma sum
                let partial_hash = self.luma_matrix.partial_luma_sum([
                    y,
                    x,
                    y + pattern.height(),
                    x + pattern.width(),
                ]);
                if partial_hash != pattern_hash {
                    continue;
                }

                for yy in 0..pattern.height() {
                    for xx in 0..pattern.width() {
                        if screenshot.pixel(x + xx, y + yy) != pattern.pixel(xx, yy) {
                            continue 'x;
                        }
                    }
                }

                return Some((x + (pattern.width() >> 1), y + (pattern.height() >> 1)));
            }
        }
        None
    }
}
