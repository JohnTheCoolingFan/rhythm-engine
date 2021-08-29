use super::preliminary::*;
use glam::{Mat3, Vec2};

pub struct PolyEntity {
    points: Vec<Vec2>, //contains position offset
    pub start: f32,
    pub duration: f32,
    pub local_center: Vec2,
    pub properties: Properties,
}

impl PolyEntity {
    #[rustfmt::skip]
    pub fn new(start: f32, duration: f32, initial: &[Vec2; 3]) -> Option<Self> {
        if Mat3::from_cols_array(&[
            initial[0].x, initial[0].y, 1.,
            initial[1].x, initial[1].y, 1.,
            initial[2].x, initial[2].y, 1.,
        ]).transpose().determinant() > 0. {
            Some(Self {
                start,
                duration,
                local_center: initial.iter().sum::<Vec2>() / initial.len() as f32,
                points: initial.to_vec(),
                properties: Properties {
                    splines: vec![],
                    rotation: vec![],
                    scale: vec![],
                    color: 0,
                    glow: 0,
                    beats: vec![]
                }
            })
        }
        else {
            None
        }
    }
}
