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
            capturer: Capturer::new(),
            simulator: Simulator::new(),
        }
    }
    pub fn capturer_mut(&mut self) -> &mut Capturer {
        &mut self.capturer
    }
    pub fn simulator_mut(&mut self) -> &mut Simulator {
        &mut self.simulator
    }
}
