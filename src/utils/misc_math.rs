use duplicate::duplicate;
use glam::Vec2;
use lyon_geom::Point;
use std::f32::consts::PI;
use std::ops::{Add, Sub, Rem};

pub trait IsLeft {
    fn is_left(&self, start: &Self, end: &Self) -> bool;
}

#[duplicate(PointT; [Point<f32>]; [Vec2])]
impl IsLeft for PointT {
    fn is_left(&self, start: &Self, end: &Self) -> bool {
        ((end.x - start.x) * (self.y - start.y) - (end.y - start.y) * (self.x - start.x)) > 0.
    }
}

pub trait RotateAbout {
    fn rotate_about(&self, pivot: &Self, angle_deg: f32) -> Self;
}

#[duplicate(PointT; [Point<f32>]; [Vec2])]
impl RotateAbout for PointT {
    fn rotate_about(&self, pivot: &Self, angle_deg: f32) -> Self {
        let c = (angle_deg * (PI / 180.)).cos();
        let s = (angle_deg * (PI / 180.)).sin();

        Self::new(
            c * (self.x - pivot.x) - s * (self.y - pivot.y) + pivot.x,
            s * (self.x - pivot.x) + c * (self.y - pivot.y) + pivot.y,
        )
    }
}

trait Quantize {
    fn quantize(&self, period: Self, offset: Self) -> Self;
}

//trying to do this with trait bounds and blanket impls gives very funky errors
#[duplicate(Num; [f32]; [i32]; [usize])]
impl Quantize for Num
{
    fn quantize(&self, period: Self, offset: Self) -> Self {
        (self - (self - offset) % period) + offset
    }
}
