use super::{State, Transition};
use crate::context::Context;
use crate::context::MouseButton;
use crate::Finder;
use crate::Screenshot;

pub enum PresetState {
    MouseMoveTo {
        pattern: Screenshot,
    },
    MouseClick {
        btn: MouseButton,
    },
    MouseClickAt {
        pattern: Screenshot,
        btn: MouseButton,
    },
    MouseScroll {
        dx: i32,
        dy: i32,
    },
    Entry,
    Exit,
}

impl State<Context> for PresetState {
    fn enter(&mut self, ctx: &mut Context) {
        println!("Enter");
        match self {
            PresetState::MouseClick { btn } => ctx.simulator_mut().mouse_click(*btn),
            PresetState::MouseScroll { dx, dy } => ctx.simulator_mut().mouse_scroll(*dx, *dy),
            _ => {}
        }
    }

    fn tick(&mut self, ctx: &mut Context) -> bool {
        println!("Tick");
        match self {
            PresetState::MouseMoveTo { pattern } => {
                let screenshot = ctx.capturer_mut().frame();
                let finder = Finder::new(&screenshot);
                if let Some(pos) = finder.find(pattern) {
                    ctx.simulator_mut().mouse_move_to(pos.0, pos.1);
                    return true;
                }
                false
            }
            PresetState::MouseClickAt { pattern, btn } => {
                let screenshot = ctx.capturer_mut().frame();
                let finder = Finder::new(&screenshot);
                if let Some(pos) = finder.find(pattern) {
                    ctx.simulator_mut().mouse_move_to(pos.0, pos.1);
                    ctx.simulator_mut().mouse_click(*btn);
                    return true;
                }
                false
            }
            _ => true,
        }
    }

    fn exit(&mut self, _ctx: &mut Context) {
        println!("Exit");
    }
}

pub enum PresetTransition {
    PatternFound { pattern: Screenshot },
    Direct,
}

impl Transition<Context, PresetState> for PresetTransition {
    fn satisfied(&self, ctx: &mut Context, _src: &PresetState, _dst: &PresetState) -> bool {
        match self {
            PresetTransition::PatternFound { pattern } => {
                let screenshot = ctx.capturer_mut().frame();
                let finder = Finder::new(&screenshot);
                finder.find(pattern).is_some()
            }
            PresetTransition::Direct => true,
        }
    }
}
