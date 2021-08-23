use std::ops::{Deref, DerefMut};
use duplicate::duplicate;
use glam::{Vec2, Mat3};
use super::Automation;
use std::marker::PhantomData;

//need this cause orphan rules
pub struct CrudeTransform<T>
where
    Mat3: From<Self>
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
    Mat3: From<CrudeTransform<T>>
{
    Pre(T, Option<Vec2>),
    Post(Mat3)
}

impl<T> Transform<T> 
where
    Mat3: From<CrudeTransform<T>>,
    T: Copy
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
    Mat3: From<CrudeTransform<T>>
{
    pub auto: Automation,
    point: Option<Vec2>,
    _phantom: PhantomData<T>
}
