/*
 * THIS NEEDS TO BE RETHOUGHT
 *
 * use crate::{
    foundation::automation::*,
    utils::{Seekable, Seeker},
};
use duplicate::duplicate;
use glam::{Mat2, Vec2};
use std::marker::PhantomData;

pub struct TransformPoint<T>
where
    T: Into<Mat2>,
    f32: Into<T>,
{
    pub point: Option<Vec2>,
    pub automation: Automation,
    _phantom: PhantomData<T>,
}

pub struct TransPointSeeker<'a, T>
where
    T: Into<Mat2>,
    f32: Into<T>,
{
    seeker: AutomationSeeker<'a>,
    tp: &'a TransformPoint<T>,
}

impl<'a, T> Seeker<Mat2> for TransPointSeeker<'a, T>
where
    T: Into<Mat2>,
    f32: Into<T>,
{
    #[duplicate(method; [seek]; [jump])]
    fn method(&mut self, val: f32) -> Mat2 {
        let t: T = self.seeker.method(val).into();
        t.into()
    }
}

impl<'a, T> Seekable<'a> for TransformPoint<T>
where
    T: Into<Mat2> + 'a,
    f32: Into<T>,
{
    type Output = Mat2;
    type SeekerType = TransPointSeeker<'a, T>;
    fn seeker(&'a self) -> Self::SeekerType {
        Self::SeekerType {
            seeker: self.automation.seeker(),
            tp: &self,
        }
    }
}*/
