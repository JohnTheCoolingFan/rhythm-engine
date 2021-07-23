use crate::utils::math::*;
use crate::utils::misc::*;
use glam::Vec2;

#[derive(Debug, Copy, Clone)]
pub enum Weight {
    ForwardBias,
    QuadLike(f32),
    CubeLike(f32),
    ReverseBias,
}

impl Weight {
    #[rustfmt::skip]
    pub fn eval(self, start: &Vec2, end: &Vec2, offset: f32) -> f32 {
        debug_assert!(start.x <= end.x, "start must be <= end");
        debug_assert!(start.x <= offset && offset <= end.x, "offset out of bounds");
        
        let mut starting = start.y;
        let mut t = (offset - start.x) / (end.x - start.x);
        let mut y_diff = end.y - start.y;
        let power = match self {
            Self::ForwardBias => return end.y,
            Self::ReverseBias => return start.y,
            Self::QuadLike(p) | Self::CubeLike(p)=> p,
        };

        if let Self::CubeLike(_) = self {
            y_diff /= 2.;
            if 0.5 <= t {
                starting = end.y;
                y_diff = -y_diff;
                t = (0.5 - t % 0.5) / 0.5;
            }
            else {
                t /= 0.5;
            }
        };
        
        starting + y_diff * t.powf(if power < 0. { 1. / (power.abs() + 1.) } else { power + 1. })
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Fancy {
    None,
    Step {
        period: f32, //0 <=
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

    pub fn interp(&self, last: &Self, input: f32) {
        debug_assert!(
            last.point.x <= self.point.x,
            "prev.anchor.point.x <= anchor.point.x"
        );
        debug_assert!(
            0. <= input && input <= 1.,
            "X val out of range in Anchor from_x call"
        );

        let (y0, y1) = match self.fancy {
            Fancy::Step { period, .. } => (input.quant_floor(period), input.quant_ceil(period)),
            _ => (last.point.y, self.point.y),
        };
    }
}
