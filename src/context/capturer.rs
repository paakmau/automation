use std::io::ErrorKind::WouldBlock;
use std::ops::Deref;
use std::thread;
use std::time::Duration;

use bytes::{Buf, BufMut};

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
            let frame = match self.capturer.frame() {
                Ok(frame) => frame,
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

            let mut flipped_bits = Vec::<u8>::with_capacity(frame.len());

            let s = w * h;
            let mut bits = frame.deref();
            for _ in 0..s {
                let bgra = bits.get_u32();
                let rgba =
                    (bgra & 0x00FF00FF) | ((bgra & 0xFF000000) >> 16) | ((bgra & 0x0000FF00) << 16);
                flipped_bits.put_u32(rgba);
            }

            return Screenshot::from_raw(w as u32, h as u32, flipped_bits).unwrap();
        }
    }
}
