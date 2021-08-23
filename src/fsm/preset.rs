use super::{State, Transition};
use crate::context::Context;
use crate::context::MouseButton;
use crate::image::Direction;
use crate::image::Finder;
use crate::image::Pattern;

pub enum PresetState<'a> {
    MouseMoveTo {
        pattern: &'a Pattern,
        dir: Direction,
    },
    MouseClick {
        btn: MouseButton,
    },
    MouseClickAt {
        pattern: &'a Pattern,
        dir: Direction,
        btn: MouseButton,
    },
    MouseScroll {
        dx: i32,
        dy: i32,
    },
    Emtpy,
    Entry,
    Exit,
}

impl<'a> State<Context> for PresetState<'a> {
    fn enter(&mut self, ctx: &mut Context) {
        match self {
            PresetState::MouseClick { btn } => ctx.simulator_mut().mouse_click(*btn),
            PresetState::MouseScroll { dx, dy } => ctx.simulator_mut().mouse_scroll(*dx, *dy),
            _ => {}
        }
    }

    fn tick(&mut self, ctx: &mut Context) -> bool {
        match self {
            PresetState::MouseMoveTo { pattern, dir } => {
                let screenshot = ctx.capturer_mut().frame();
                let finder = Finder::new(&screenshot);
                if let Some(pos) = finder.find(pattern, *dir) {
                    ctx.simulator_mut().mouse_move_to(pos.0, pos.1);
                    return true;
                }
                false
            }
            PresetState::MouseClickAt { pattern, dir, btn } => {
                let screenshot = ctx.capturer_mut().frame();
                let finder = Finder::new(&screenshot);
                if let Some(pos) = finder.find(pattern, *dir) {
                    ctx.simulator_mut().mouse_move_to(pos.0, pos.1);
                    ctx.simulator_mut().mouse_click(*btn);
                    return true;
                }
                false
            }
            _ => true,
        }
    }

    fn exit(&mut self, _ctx: &mut Context) {}
}

pub enum PresetTransition<'a> {
    PatternFound {
        pattern: &'a Pattern,
        dir: Direction,
    },
    Direct,
}

impl<'a> Transition<Context, PresetState<'a>> for PresetTransition<'a> {
    fn satisfied(&self, ctx: &mut Context, _src: &PresetState, _dst: &PresetState) -> bool {
        match self {
            PresetTransition::PatternFound { pattern, dir } => {
                let screenshot = ctx.capturer_mut().frame();
                let finder = Finder::new(&screenshot);
                finder.find(pattern, *dir).is_some()
            }
            PresetTransition::Direct => true,
        }
    }
}
