use duplicate::duplicate;
use glam::Vec2;
use lyon_geom::Point;

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

pub trait FloatUtils {
    fn quant_floor(&self, period: Self, offset: Self) -> Self;
}

impl FloatUtils for f32 {
    fn quant_floor(&self, period: Self, offset: Self) -> Self {
        if period == 0. {
            *self
        }
        else {
            ((self - offset) / period).floor() * period + offset
        }
    }
}
