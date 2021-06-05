use glam::Vec2;
use crate::utils::seeker::*;

pub struct Anchor {
    point: Vec2,
    power: f32
}

pub struct Automation {
    upper_bound: f32,
    lower_bound: f32,
    dynamic_bound: bool,

    anchors: Vec<Anchor>,
}

impl Automation {
    pub fn new(ub: f32, lb: f32, len: f32, dynamic: bool) -> Self {
        Automation {
            upper_bound: ub,
            lower_bound: lb,
            dynamic_bound: dynamic,
            anchors: vec![
                Anchor{ point: Vec2::new(0., 0.0), power: 1. },
                Anchor{ point: Vec2::new(len, 0.0), power: 1. }
            ]
        }
    }

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

    pub fn push(&mut self, anchor: Anchor) {
        self.anchors.push(anchor);
    }

    pub fn pop(&mut self) -> Anchor {
        self.anchors.pop().unwrap()
    }

    pub fn remove(&mut self, index: usize) -> Anchor {
        self.anchors.remove(index)
    }
}

pub struct AutomationSeeker<'a> {
    index: usize,
    automantion: &'a Automation
}

impl <'a> AutomationSeeker<'a> {
    fn y_to_lbub_val(&self, y: f32) -> f32 {
        debug_assert!(0. <= y && y <= 1.);
        self.automantion.lower_bound + (self.automantion.upper_bound - self.automantion.lower_bound) * y
    }

    fn interp(&self, offset: f32) -> f32 {
        debug_assert!(0 < self.index && self.index < self.automantion.anchors.len());
        debug_assert!(
            self.automantion.anchors[self.index - 1].point.x <= offset 
            && offset <= self.automantion.anchors[self.index].point.x
        );

        let start = &self.automantion.anchors[self.index - 1];
        let end = &self.automantion.anchors[self.index];
        
        let t = (offset - self.automantion.anchors[self.index - 1].point.x)
            / (self.automantion.anchors[self.index].point.x - self.automantion.anchors[self.index - 1].point.x);

        self.y_to_lbub_val(start.point.y + (end.point.y - start.point.y) * t.powf(end.power))
    }
}

impl <'a> Seeker<f32> for AutomationSeeker<'a> {
    fn seek(&mut self, offset: f32) -> f32 {
        while self.index < self.automantion.anchors.len() {
            if offset == self.automantion.anchors[self.index].point.x {
                return self.y_to_lbub_val(self.automantion.anchors[self.index].point.y);
            }
            else if offset < self.automantion.anchors[self.index].point.x {
                break;
            }
            self.index += 1;
        }
        if 0 == self.index {
            self.y_to_lbub_val(self.automantion.anchors[0].point.y)
        }
        else {
            self.interp(offset)
        }
        
    }

    fn jump(&mut self, offset: f32) -> f32 {
        match self.automantion.anchors.binary_search_by(|t| t.point.x.partial_cmp(&offset).unwrap()) {
            Ok(index) => {
                self.index = index;
                self.y_to_lbub_val(self.automantion.anchors[index].point.y)
            }
            Err(index) => {
                self.index = index;
                if 0 == index || index == self.automantion.anchors.len() {
                    self.y_to_lbub_val(self.automantion.anchors[index].point.y)
                }
                else {
                    self.interp(offset)
                }
            }
        }
    }
}
