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

    pub fn interp(&self, last: &Self, offset: f32) -> f32 {
        debug_assert!(last.point.x <= self.point.x, "self < last");
        debug_assert!(
            last.point.x <= offset && offset <= self.point.x,
            "offset out of bounds"
        );

        let length = self.point.x - last.point.x;
        let height = self.point.y - last.point.y;
        let macro_t = (offset - last.point.x) / (self.point.x - last.point.y);

        match self.embelish {
            Some((fancy, subwave)) => {
                let (x0, x1) = (
                    offset
                        .quant_floor(subwave.period, subwave.offset)
                        .clamp(last.point.x, self.point.x),
                    offset
                        .quant_ceil(subwave.period, subwave.offset)
                        .clamp(last.point.x, self.point.x),
                );

                let mini_t = (offset - x0) / (x1 - x0);

                match fancy {
                    Fancy::Step => {
                        let (y0, y1) = (
                            self.weight.eval(x0 / length) * height,
                            self.weight.eval(x1 / length) * height,
                        );

                        last.point.y + y0 + (y1 - y0) * subwave.inner.eval(mini_t)
                    }
                    Fancy::Oscilate { alternate } => {
                        let (y0, y1) = (
                            self.weight.eval(x0 / length) * 0.5 * height,
                            self.weight.eval(x1 / length) * 0.5 * height,
                        );

                        let delta = if alternate && (offset - subwave.offset / subwave.period) as i32 % 2 == 1 {

                        }
                    }
                }
            }
            None => last.point.y + height * self.weight.eval(macro_t),
        }
    }
}
