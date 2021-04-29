use glam::Vec2;
use duplicate::duplicate;
use crate::utils::from_end::FromEnd;

pub struct AutomationSeekCache {
    index: usize
}

pub struct Automation {
    upper_bound: f32,
    lower_bound: f32,
    dynamic_bound: bool,
    points: Vec<Vec2>,
    
    seek_cahce: AutomationSeekCache
}

impl Automation
{
    pub fn get_upper_bound(&self) { self.upper_bound }
    pub fn get_lower_bound(&self) { self.lower_bound }

    pub fn set_upper_bound(&mut self, v: f32) { 
        self.dynamic_bound.then(|| self.upper_bound = v);
    }
    pub fn set_lower_bound(&mut self, v: f32) { 
        self.dynamic_bound.then(|| self.lower_bound = v);
    }

    fn lerp(&self, index: usize/*starting point*/, amount: f32 /*0.0 to 1.0*/) {
        let span = self.upper_bound - self.lower_bound;
        if index < self.points.len() - 1 {
            self.points[index].lerp(self.points[index + 1], amount) * span
        }
        else { 
            self.points[FromEnd(0)].y() * span
        }
    }

    pub fn start(&mut self, offset: f32) {
        let mut i = &self.seek_cahce.index;
        for i in (0..self.points.len()).step_by(2) {
            if offset < self.points[i].x() {

            }
    }
}
