use std::io::ErrorKind::WouldBlock;
use std::thread;
use std::time::Duration;

use crate::image::Screenshot;

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

            return Screenshot::from_bgra_buf(w as u32, h as u32, frame.to_vec()).unwrap();
        }
    }
}

impl Default for Capturer {
    fn default() -> Self {
        Self::new()
    }
}
