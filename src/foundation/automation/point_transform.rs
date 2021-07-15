use crate::{
    foundation::automation::*,
    utils::{Seekable, Seeker},
};
use glam::{Mat2, Vec2};

pub struct RotationPoint {
    pub point: Option<Vec2>,
    pub automation: Automation,
}

impl RotationPoint {
    pub fn new(len: f32) -> Self {
        Self {
            point: None,
            automation: Automation::new(0., 360., len),
        }
    }
}

pub struct RotPSeeker<'a> {
    seeker: AutomationSeeker<'a>,
    rp: &'a RotationPoint,
}

impl<'a> Seeker<Mat2> for RotPoint<'a> {
    fn seek(&mut self, val: f32) -> Mat2 {
        (self.pt.point, self.seeker.seek(val))
    }

    fn jump(&mut self, val: f32) -> Mat2 {
        (self.pt.point, self.seeker.jump(val))
    }
}

impl<'a> Seekable<'a> for PointTransform {
    type Output = (Option<Vec2>, f32);
    type SeekerType = PTSeeker<'a>;
    fn seeker(&'a self) -> Self::SeekerType {
        Self::SeekerType {
            seeker: self.automation.seeker(),
            pt: &self,
        }
    }
}
