use lyon_geom::Point;
use glam::Vec2;
use std::f32::consts::PI;
use duplicate::duplicate;

pub trait IsLeft {
    fn is_left(&self, start: &Self, end: &Self) -> bool;
}


#[duplicate(PointT; [Point<f32>]; [Vec2])]
impl IsLeft for PointT {
    fn is_left(&self, start: &Self, end: &Self) -> bool {
        (
            (end.x - start.x)*(self.y - start.y) - 
            (end.y - start.y)*(self.x - start.x)
        ) > 0.
    }
}

pub trait RotateAbout {
    fn rotate_about(&self, pivot: &Self, angle_deg: f32) -> Self;
}

#[duplicate(PointT; [Point<f32>]; [Vec2])]
impl RotateAbout for PointT {
    fn rotate_about(&self, pivot: &Self, angle_deg: f32) -> Self {
        let c = (angle_deg * PI / 180.).cos(); 
        let s = (angle_deg * PI / 180.).sin();

        let x = self.x - pivot.x;
        let y = self.y - pivot.y;
        
        Self::new(
            (c * x) - (s * y) + pivot.x,
            (s * x) - (c * y) + pivot.y
        )
    }
}
