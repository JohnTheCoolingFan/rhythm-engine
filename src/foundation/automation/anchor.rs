use crate::utils::math::*;
use crate::utils::misc::*;
use glam::Vec2;

#[derive(Debug, Copy, Clone)]
pub enum Weight {
    ForwardBias,
    Quad(f32),
    Cube(f32),
    ReverseBias,
}

#[derive(Debug, Copy, Clone)]
pub enum Fancy {
    None,
    Step {
        period: f32, //-inf to inf
        inner: Weight,
    },
    Oscilate {
        offset: f32, //0 <=
        period: f32, //0 <=
        alternate: bool,
    },
}

pub struct Anchor {
    pub point: Vec2,
    pub weight: Weight,
    pub fancy: Fancy,
}

impl Anchor {
    pub fn new(p: Vec2, w: Weight) -> Self {
        Self {
            point: p,
            weight: w,
            fancy: Fancy::None,
        }
    }

    pub fn from_x(&self, start: f32, mut x: f32) {
        debug_assert!(
            start <= self.point.x,
            "prev.anchor.point.x <= anchor.point.x"
        );
        debug_assert!(
            0. <= x && x <= 1.,
            "X val out of range in Anchor from_x call"
        );

        let x = match self.fancy {
            Fancy::Oscilate {
                period,
                offset,
                alternate,
            } => x,
            _ => x,
        };

        match self.fancy {}
    }
}
