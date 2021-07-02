use super::Automation;
use glam::Vec2;

pub struct PointTransform {
    pub point: Option<Vec2>,
    pub automation: Automation
}

impl PointTransform {
    pub fn new(len: f32) -> Self {
        Self {
            point: None,
            automation: Automation::new(0., 360., len)
        }
    }
}
