use std::io::ErrorKind::WouldBlock;
use std::ops::Deref;
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
            let bgra_buf = frame.deref();

            let mut rgba_buf = vec![0u8; bgra_buf.len()];
            for i in (0..bgra_buf.len()).step_by(4) {
                let gbra = &bgra_buf[i..i + 4];
                rgba_buf[i..i + 4].copy_from_slice(&[gbra[2], gbra[1], gbra[0], gbra[3]]);
            }

            return Screenshot::from_raw(w as u32, h as u32, rgba_buf).unwrap();
        }
    }
}
