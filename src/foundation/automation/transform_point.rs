use std::ops::{Deref, DerefMut};
use duplicate::duplicate;
use glam::{Vec2, Mat2};
use super::Automation;

pub struct Rotation(pub f32);
pub struct Scale(pub f32);

#[duplicate(T; [Rotation]; [Scale])]
impl Deref for T {
    type Target = f32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[duplicate(T; [Rotation]; [Scale])]
impl DerefMut for T {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub enum Transform<T>
where
    Mat2: From<T>
{
    PreTrans(T, Option<Vec2>),
    Trans(Mat2)
}

impl<T> Transform<T> 
where
    Mat2: From<T>
{
    pub fn process(&mut self, auxiliary: Vec2) {
        if let Self::PreTrans(factor, point) = self {
            *self = Self::Trans(factor.into())
        }
    }
}

struct TransformPoint<T>
where
    Mat2: From<T>,
{
    pub auto: Automation,
    point: Option<Vec2>
}


