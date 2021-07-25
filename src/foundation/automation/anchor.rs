use crate::utils::math::*;
use crate::utils::misc::*;
use glam::Vec2;

#[derive(Debug)]
pub enum Weight {
    ForwardBias,
    QuadLike(f32),
    CubeLike(f32),
    ReverseBias,
}

impl Weight {
    #[rustfmt::skip]
    pub fn eval(&self, t: f32) -> f32 {
        debug_assert!(0. <= t && t <= 1., "t out of bounds");

        match self {
            Self::ForwardBias => 1.,
            Self::ReverseBias => 0.,
            Self::QuadLike(power) | Self::CubeLike(power) => {
                //cubic is basically 2 quadratics with the 2nd
                //being inverted about the half way point
                let (starting, delta, x) = if let Self::CubeLike(_) = self {
                    if 0.5 < t {
                        (
                            1.,
                            -0.5,
                            (0.5 - t % 0.5) / 0.5
                        )
                    } else {
                        (
                            0.,
                            0.5,
                            t / 0.5
                        )
                    }
                } else {
                    (
                        0.,
                        1.,
                        t
                    )
                };

                starting + delta * x.powf(
                    if power < 0. {
                        1. / (power.abs() + 1.)
                    } else {
                        power + 1.
                    }
                )
            }
        }
    }
}

#[derive(Debug)]
pub struct SubWave {
    pub offset: f32,
    pub period: f32,
    pub wave: Weight,
}

#[derive(Debug)]
pub enum Fancy {
    Step,
    Hop { alternate: bool },
    Oscilate { alternate: bool },
}

#[derive(Debug)]
pub struct Anchor {
    pub point: Vec2,
    pub weight: Weight,
    pub embelish: Option<(Fancy, SubWave)>,
}

impl Anchor {
    pub fn new(p: Vec2, w: Weight) -> Self {
        debug_assert!(0. <= p.y && p.y <= 1., "anchor point y out of range");
        Self {
            point: p,
            weight: w,
            embelish: None,
        }
    }


    #[rustfmt::skip]
    pub fn interp(&self, last: &Self, offset: f32) -> f32 {
        debug_assert!(last.point.x <= self.point.x, "self < last");
        debug_assert!(
            last.point.x <= offset && offset <= self.point.x,
            "offset out of bounds"
        );

        let dy = self.point.y - last.point.y;

        let (x0, x1) = if let Some((fancy, _)) = self.fancy {
            (
                offset
                    .quant_floor(fancy.period, fancy.offset)
                    .clamp(last.point.x, self.point.x),
                offset
                    .quant_ceil(fancy.period, fancy.offset)
                    .clamp(last.point.x, self.point.x),
            )
        } else {
            (last.point.x, self.point.x)
        };

        let (y0, y1) = if let Some((fancy, _)) = self.fancy {
            match fancy {
                Fancy::Step => (
                    last.point.y + self.weight.eval(x0) * dy,
                    last.point.y + self.weight.eval(x1) * dy
                ),
                Fancy::Hop { .. } => (
                    last.point.y,
                    last.point.y + self.weight.eval(x1) * dy
                ),
                Fancy::Oscilate { .. } => (
                    (last.point.y + self.weight.eval(x0) * dy) / 2.
                    (self.point.y - self.weight.eval(x1) * dy) / 2.
                ),
            }
        } else {
            (last.point.y, self.point.y)
        };
    }
}
