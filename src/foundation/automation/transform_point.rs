use crate::{
    foundation::automation::*,
    utils::{Seekable, Seeker},
};
use glam::{Mat2, Vec2};

pub struct TransformPoint<T>
where
    T: Into<Mat2>,
    f32: Into<T>,
{
    pub point: Option<Vec2>,
    pub automation: Automation,
}

pub struct TransPointSeeker<'a, T> {
    seeker: AutomationSeeker<'a>,
    rp: &'a TransformPoint<T>,
}

impl<'a, T> Seeker<Mat2> for TransPointSeeker<'a, T> {
    fn seek(&mut self, val: f32) -> Mat2 {
        self.seeker.seek(val).into::<T>().into::<Mat2>()
    }
    fn jump(&mut self, val: f32) -> Mat2 {
        self.seeker.jump(val).into::<T>().into::<Mat2>()
    }
}

impl<'a, T> Seekable<'a> for TransformPoint<T> {
    type Output = (Option<Vec2>, f32);
    type SeekerType = TransPointSeeker<'a, T>;
    fn seeker(&'a self) -> Self::SeekerType {
        Self::SeekerType {
            seeker: self.automation.seeker(),
            pt: &self,
        }
    }
}
