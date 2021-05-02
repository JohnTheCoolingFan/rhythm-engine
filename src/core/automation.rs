use crate::utils::from_end::FromEnd;
use crate::core::curves::*;
use glam::Vec2;
pub struct AutomationSeekCache {
    index: usize,
}

pub struct Automation {
    upper_bound: f32,
    lower_bound: f32,
    dynamic_bound: bool,
    curve: CurveChain,

    seek_cahce: AutomationSeekCache,
}

impl Automation {
    pub fn get_upper_bound(&self) -> f32 {
        self.upper_bound
    }
    pub fn get_lower_bound(&self) -> f32 {
        self.lower_bound
    }

    pub fn set_upper_bound(&mut self, v: f32) {
        if self.dynamic_bound { self.upper_bound = v; }
    }
    pub fn set_lower_bound(&mut self, v: f32) {
        if self.dynamic_bound { self.lower_bound = v; }
    }
}
