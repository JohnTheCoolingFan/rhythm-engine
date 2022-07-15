use super::automation::Weight;
use crate::utils::*;
use bevy::prelude::*;
use noisy_float::prelude::*;

pub struct RepeaterClamp {
    start: T32,
    end: T32,
    weight: Weight,
}

impl RepeaterClamp {
    pub fn eval(&self, t: T32) -> T32 {
        self.start.lerp(&self.end, self.weight.eval(t))
    }
}

#[derive(Component)]
pub struct Repeater {
    run_time: R32,
    ping_pong: bool,
    ceil: RepeaterClamp,
    floor: RepeaterClamp,
}
