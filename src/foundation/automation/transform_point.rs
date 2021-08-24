use std::ops::{Deref, DerefMut};
use duplicate::duplicate;
use glam::{Vec2, Mat3};
use super::{automation::*, anchor::*};
use crate::utils::seeker::*;
use std::marker::PhantomData;

pub trait TransformDictator: Copy + Deref<Target = f32> + From<f32> {}
impl<T> TransformDictator for T 
where
    T: Copy + Deref<Target = f32> + From<f32>
{}

//need this cause orphan rules
pub struct CrudeTransform<T>
where
    Mat3: From<Self>,
    T: TransformDictator
{
    pub factor: T,
    pub pivot: Vec2
}

#[derive(Clone, Copy)]
pub struct Rotation(pub f32);
#[derive(Clone, Copy)]
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

#[duplicate(T; [Rotation]; [Scale])]
impl From<f32> for T {
    fn from(v: f32) -> Self {
        Self(v)
    }
}

impl From<CrudeTransform<Rotation>> for Mat3 {
    #[rustfmt::skip]
    fn from(CrudeTransform{ factor, pivot }: CrudeTransform<Rotation>) -> Self {
        let r = factor.to_radians();
        let (x, y) = (pivot.x, pivot.y);
        let (r10, r11) = r.sin_cos();
        let (r00, r01) = (r11, -r10);

        Mat3::from_cols_array(&[
            r00,    r01,    x - r00 * x - r01 * y,
            r10,    r11,    y - r10 * x - r11 * y,
            0.,      0.,    1.,
        ]).transpose()
    }
}

impl From<CrudeTransform<Scale>> for Mat3 {
    #[rustfmt::skip]
    fn from(CrudeTransform{ factor, pivot }: CrudeTransform<Scale>) -> Self {
        let s = factor.0;
        Mat3::from_cols_array(&[
            s,      0.,     pivot.x - s * pivot.x,
            0.,     s,      pivot.y - s * pivot.y,
            0.,     0.,     1.
        ])
    }
}
//
//
//
//
//
pub enum Transform<T>
where
    Mat3: From<CrudeTransform<T>>,
    T: TransformDictator
{
    Pre(T, Option<Vec2>),
    Post(Mat3)
}

impl<T> Transform<T> 
where
    Mat3: From<CrudeTransform<T>>,
    T: TransformDictator
{
    pub fn process(&mut self, auxiliary: &Vec2) -> &Mat3 {
        match self {
            Self::Pre(factor, point) =>
                *self = Self::Post(CrudeTransform{factor:*factor, pivot:
                    if let Some(p) = point { *p }
                    else { *auxiliary }
                }.into()),
            Self::Post(ref transform) => return transform
        }
        self.process(auxiliary)
    }
}

struct TransformPoint<T>
where
    Mat3: From<CrudeTransform<T>>,
    T: TransformDictator
{
    pub auto: Automation,
    pub point: Option<Vec2>,
    _phantom: PhantomData<T>
}
//
//
//
//
//
pub type TransformPointSeeker<'a, T> = Seeker<(Option<Vec2>, PhantomData<T>), AutomationSeeker<'a>>;

impl<'a, T> SeekerTypes for TransformPointSeeker<'a, T>
where
    Mat3: From<CrudeTransform<T>>,
    T: TransformDictator
{
    type Source = Anchor;
    type Output = Transform<T>;
}

impl<'a, T> Seek for TransformPointSeeker<'a, T>
where
    Mat3: From<CrudeTransform<T>>,
    T: TransformDictator
{
    #[duplicate(method; [seek]; [jump])]
    fn method(&mut self, offset: f32) -> Transform<T> {
        let (point, _) = self.data;
        Transform::<T>::Pre(
            self.meta.method(offset).into(),
            point
        )
    }
}

impl <'a, T> Seekable<'a> for TransformPoint<T>
where
    Mat3: From<CrudeTransform<T>>,
    T: TransformDictator
{
    type Seeker = TransformPointSeeker<'a, T>;
    fn seeker(&'a self) -> Self::Seeker {
        Self::Seeker{
            data: (self.point, PhantomData),
            meta: self.auto.seeker()
        }
    }
}
//
//
//
//
//
#[cfg(test)]
mod tests {
    use super::*;
    use ggez::{
        event::{self, EventHandler, MouseButton, KeyCode},
        graphics::*,
        Context,
        GameError,
        GameResult
    };
    struct Test {
        rotation: TransformPoint<Rotation>,
        scale: TransformPoint<Scale>,
        dimensions: Vec2
    }

    impl Test {
        fn new(len: f32) -> GameResult<Self> {
            Ok(Self{
                rotation: TransformPoint<Rotation>{

                },
                scale: 
            })
        }
    }

    pub fn transform_point() -> GameResult {
        
    }
}
