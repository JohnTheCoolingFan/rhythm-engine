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

use crate::foundation::*;
use ggez::graphics::Color;

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
    },
    Hold {
        pre: f32,
        follow: Automation<f32>, //attack: start.x, post: end.x
        keys: Option<HitKeys>,
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
pub struct CSVPairing {
    pub spline: usize,
    pub vertex: usize,
    pub scale: Reference<Scale>,
    pub rotation: Reference<Rotation>,
    pub x_invert: bool,
    pub y_invert: bool,
}

pub struct Properties {
    pub point_shifts: Vec<CSVPairing>,
    pub rotation: Vec<Controller<Rotation>>,
    pub scale: Vec<Controller<Scale>>,
    pub color: Controller<Color>,
    pub bloom: Controller<f32>,
    pub beats: Vec<Beat>,
}

