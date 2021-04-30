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
    flattened: Vec<Vec<Vec2>>,
    
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

    fn lerp(&self, index: usize/*starting point*/, offset: f32 ) {
        let span = self.upper_bound - self.lower_bound;
        let points = &self.points;
    }

    pub fn start(&mut self, offset: f32) {
        for i in (0..self.points.len()).step_by(2) {
            if offset < self.points[i].x() {
                self.seek_cahce.index = i;
                self.lerp(i, offset)
            }
        }
        self.seek_cahce.index = self.points.len() - 1;
        self.lerp(self.points.len() - 1, offset)
    }

    pub fn advance(&mut self, offset: f32) {
        for i in (self.seek_cahce.index..self.points.len()).step_by(2) {
            if offset < self.points[i].x() {
                self.seek_cahce.index = i;
                self.lerp(i, offset)
            }
        }
        self.seek_cahce.index = self.points.len() - 1;
        self.lerp(self.points.len() - 1, offset)
    }
}
