use glam::{Mat3, Vec2};
use ggez::graphics::Color;
use crate::automation::*;
/*use glam::Vec2;

trait PolygonExtensions {
    type VecT;
    fn clockwise(&self) -> bool;
    fn break_at_intersects(&self) -> Vec<Vec<Self::VecT>>;
}


impl PolygonExtensions for &[Vec2] {
    type VecT = Vec2;
    fn clockwise(&self) -> bool {
        0.0_f32 < self.iter()
            .skip(1)
            .enumerate()
            .map(|(i, p)| {
                let prev = self[i - 1];
                (p.x - prev.x) * (p.y + prev.y)
            })
            .sum()
    }

    fn triangulate(&self) -> {
        
    }
}*/


pub struct HitKeys {
    alphas: u8,
    phat: bool,
}

pub enum Beat {
    //0. <= pre <= 1.
    //pre + attack = activation time
    //pre + post = release time
    //no keys == lazy hit
    Hit {
        pre: f32,
        attack: f32,
        keys: Option<HitKeys>,
        layer: u8
    },
    Hold {
        pre: f32,
        follow: Automation<f32>, //attack: start.x, post: end.x
        keys: Option<HitKeys>,
        layer: u8
    },
    Avoid {
        pre: f32,
        attack: f32,
        post: f32,
    },
}

pub enum Reference<T> {
    Relative(T),
    Absolute(T)
}

pub enum Controller<T> {
    Static(T),
    Automated(usize)
}

//Complex Spline Vertex Pairing
pub struct CsvPairing {
    pub spline: usize,
    pub vertex: usize,
    pub scale: Reference<Scale>,
    pub rotation: Reference<Rotation>,
    pub x_invert: bool,
    pub y_invert: bool,
}

pub struct Properties {
    pub point_shifts: Vec<CsvPairing>,
    pub rotation: Vec<Controller<Rotation>>,
    pub scale: Vec<Controller<Scale>>,
    pub color: Controller<Color>,
    pub bloom: Controller<f32>,
    pub beats: Vec<Beat>,
}

pub struct PolyEntity {
    pub points: Vec<Vec2>, //contains position offset
    pub start: f32,
    pub duration: f32,
    pub properties: Properties,
}
//
//
//
//
//
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
                points: {
                    let mut controls = vec![
                        initial.iter().sum::<Vec2>() / initial.len() as f32,    //local center
                        Vec2::new(0., 0.)                                       //position
                    ];
                    controls.extend_from_slice(initial);
                    controls
                },
                properties: Properties {
                    point_shifts: vec![],
                    rotation: vec![],
                    scale: vec![],
                    color: Controller::Static(Color::WHITE),
                    bloom: Controller::Static(0.),
                    beats: vec![]
                }
            })
        }
        else {
            None
        }
    }

}
