use crate::{foundation::automation::*, utils::{Seekable, Seeker}};
use glam::Vec2;

pub struct PointTransform {
    pub point: Option<Vec2>,
    pub automation: Automation
}

impl PointTransform {
    pub fn new(lb: f32, ub: f32, len: f32) -> Self {
        Self {
            point: None,
            automation: Automation::new(lb, ub, len)
        }
    }
}

pub struct PTSeeker<'a> {
    seeker: AutomationSeeker<'a>,
    pt: &'a PointTransform
}

impl<'a> Seeker<(Option<Vec2>, f32)> for PTSeeker<'a> {
    fn seek(&mut self, val: f32) -> (Option<Vec2>, f32) {
        (self.pt.point, self.seeker.seek(val))
    }

    fn jump(&mut self, val: f32) -> (Option<Vec2>, f32) {
        (self.pt.point, self.seeker.jump(val))
    }
}

impl<'a> Seekable<'a> for PointTransform {
    type Output = (Option<Vec2>, f32);
    type SeekerType = PTSeeker<'a>;
    fn seeker(&'a self) -> Self::SeekerType {
        Self::SeekerType {
            seeker: self.automation.seeker(),
            pt: &self
        }
    }
}
