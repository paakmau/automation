mod capturer;
mod simulator;

pub use capturer::*;
pub use simulator::*;

pub struct Context {
    capturer: Capturer,
    simulator: Simulator,
}

impl Context {
    pub fn new() -> Context {
        Context {
            capturer: Default::default(),
            simulator: Default::default(),
        }
    }
    pub fn capturer_mut(&mut self) -> &mut Capturer {
        &mut self.capturer
    }
    pub fn simulator_mut(&mut self) -> &mut Simulator {
        &mut self.simulator
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}
