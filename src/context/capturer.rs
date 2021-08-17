use std::io::ErrorKind::WouldBlock;
use std::thread;
use std::time::Duration;

use crate::Screenshot;

pub struct Capturer {
    capturer: scrap::Capturer,
}

impl Capturer {
    pub fn new() -> Self {
        let display = scrap::Display::primary().expect("Failed to find primary display.");
        let capturer = scrap::Capturer::new(display).expect("Failed to begin capture.");
        Self { capturer }
    }

    pub fn screen_size(&self) -> (u32, u32) {
        (self.capturer.width() as u32, self.capturer.height() as u32)
    }

    pub fn frame(&mut self) -> Screenshot {
        let one_second = Duration::new(1, 0);
        let one_frame = one_second / 60;

        let (w, h) = (self.capturer.width(), self.capturer.height());

        loop {
            // Wait until there's a frame.

            let buffer = match self.capturer.frame() {
                Ok(buffer) => buffer,
                Err(error) => {
                    if error.kind() == WouldBlock {
                        // Keep spinning.
                        thread::sleep(one_frame);
                        continue;
                    } else {
                        panic!("Error: {}", error);
                    }
                }
            };

            // Flip the ABGR image into a RGB image.
            let mut flipped_bits = Vec::with_capacity(w * h * 3);
            let stride = buffer.len() / h;
            for y in 0..h {
                for x in 0..w {
                    let i = stride * y + 4 * x;
                    flipped_bits.extend_from_slice(&[buffer[i + 2], buffer[i + 1], buffer[i]]);
                }
            }

            return Screenshot::from_raw(w as u32, h as u32, flipped_bits).unwrap();
        }
    }
}
