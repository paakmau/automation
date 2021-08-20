use tfc::{Context, MouseContext};

#[derive(Clone, Copy, Debug)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

pub struct Simulator {
    context: Context,
}

impl Simulator {
    pub fn new() -> Self {
        let context = Context::new().expect("Failed to get context");
        Self { context }
    }

    pub fn mouse_move_to(&mut self, x: u32, y: u32) {
        self.context
            .mouse_move_abs(x as i32, y as i32)
            .expect("Failed to simulate mouse moving");
    }

    pub fn mouse_move_by(&mut self, dx: i32, dy: i32) {
        self.context
            .mouse_move_rel(dx, dy)
            .expect("Failed to simulate mouse moving");
    }

    pub fn mouse_click(&mut self, btn: MouseButton) {
        self.context
            .mouse_click(match btn {
                MouseButton::Left => tfc::MouseButton::Left,
                MouseButton::Middle => tfc::MouseButton::Middle,
                MouseButton::Right => tfc::MouseButton::Right,
            })
            .expect("Failed to simulate mouse click");
    }

    pub fn mouse_scroll(&mut self, dx: i32, dy: i32) {
        self.context.mouse_scroll(dx, dy).expect("Failed to simulate mouse scrolling");
    }
}
