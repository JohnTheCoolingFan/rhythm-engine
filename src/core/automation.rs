use crate::core::curves::*;
use crate::utils::from_end::FromEnd;
use glam::Vec2;
use lyon_geom::Point;
use std::marker::PhantomData;

pub struct AutomationSeekCache {
    index: usize,
}

pub struct Automation {
    offset: f32,

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
        if self.dynamic_bound {
            self.upper_bound = v;
        }
    }
    pub fn set_lower_bound(&mut self, v: f32) {
        if self.dynamic_bound {
            self.lower_bound = v;
        }
    }

    /*pub fn new(_offset: f32, ub: f32, lb: f32, len: f32) {
        Automation {
            offset: _offset,
            upper_bound: ub,
            lower_bound: lb,
            _bound_t: PhantomData,
            curve: CurveChain
        }
    }*/
}
