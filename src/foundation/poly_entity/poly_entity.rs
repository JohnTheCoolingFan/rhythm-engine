use super::preliminary::*;
use glam::{Mat3, Vec2};

pub struct PolyEntity {
    pub points_raw: Vec<Vec2>, //contains position offset
    pub points_processed: Vec<Vec2>,
    pub start: f32,
    pub duration: f32,
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
                points_raw: {
                    let mut controls = vec![
                        initial.iter().sum::<Vec2>() / initial.len() as f32,    //local center
                        Vec2::new(0., 0.)                                       //position
                    ];
                    controls.extend_from_slice(initial);
                    controls
                },
                points_processed: vec![],
                properties: Properties {
                    point_shifts: vec![],
                    rotation: vec![],
                    scale: vec![],
                    color: Controller::Automated(0),
                    bloom: Controller::Automated(0),
                    beats: vec![]
                }
            })
        }
        else {
            None
        }
    }
}
