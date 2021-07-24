use duplicate::duplicate;
use glam::{Mat2, Vec2};
use lyon_geom::Point;
use std::f32::consts::PI;

pub trait IsLeft {
    fn is_left(&self, start: &Self, end: &Self) -> bool;
}

#[duplicate(PointT; [Point::<f32>]; [Vec2])]
impl IsLeft for PointT {
    fn is_left(&self, start: &Self, end: &Self) -> bool {
        ((end.x - start.x) * (self.y - start.y) - (end.y - start.y) * (self.x - start.x)) > 0.
    }
}

pub trait RotateAbout {
    fn rotate_about(&self, pivot: &Self, angle_deg: f32) -> Self;
}

#[duplicate(PointT; [Point::<f32>]; [Vec2])]
impl RotateAbout for PointT {
    fn rotate_about(&self, pivot: &Self, angle_deg: f32) -> Self {
        let c = angle_deg.to_radians().cos();
        let s = angle_deg.to_radians().sin();

        Self::new(
            c * (self.x - pivot.x) - s * (self.y - pivot.y) + pivot.x,
            s * (self.x - pivot.x) + c * (self.y - pivot.y) + pivot.y,
        )
    }
}

pub trait ScaleAbout {
    fn scale_about(&self, pivot: &Self, factor: f32) -> Self;
}

#[duplicate(PointT; [Point::<f32>]; [Vec2])]
impl ScaleAbout for PointT {
    fn scale_about(&self, pivot: &Self, factor: f32) -> Self {
        let p = *self - *pivot;
        PointT::new(p.x * factor + pivot.x, p.y * factor + pivot.y)
    }
}

pub trait Quantize {
    fn quant_floor(&self, period: Self) -> Self;
    fn quant_ceil(&self, period: Self) -> Self;
}

//trying to do this with trait bounds and blanket impls gives very funky errors
#[duplicate(Num; [f32]; [i32]; [usize])]
impl Quantize for Num {
    fn quant_floor(&self, period: Self) -> Self {
        self / period
    }
    fn quant_ceil(&self, period: Self) -> Self {
        let mut q = self.quant_floor(period);
        if (0 as Num) < self - q {
            q += period
        }

        q
    }
}

pub struct Rotation(f32);
pub struct Scale(f32);

#[duplicate(NT; [Rotation]; [Scale])]
impl Into<NT> for f32 {
    fn into(self) -> NT {
        NT(self)
    }
}

impl Into<Mat2> for Rotation {
    #[rustfmt::skip]
    fn into(self) -> glam::Mat2 {
        let c = self.0.to_radians().cos();
        let s = self.0.to_radians().sin();
        Mat2::from_cols_array(&[
            c, -s,
            s, c
        ]).transpose()
    }
}

impl Into<Mat2> for Scale {
    #[rustfmt::skip]
    fn into(self) -> Mat2 {
        Mat2::from_cols_array(&[
            self.0, 0.,
            0., self.0 
        ])
    }
}
