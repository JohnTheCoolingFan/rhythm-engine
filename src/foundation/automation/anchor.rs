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
                    if 0.5 < t {(
                        1.,
                        -0.5,
                        (0.5 - t % 0.5) / 0.5
                    )} else {(
                        0.,
                        0.5,
                        t / 0.5
                    )}
                } else {(
                    0.,
                    1.,
                    t
                )};

                starting + delta * x.powf(
                    if *power < 0. {
                        1. / (power.abs() + 1.)
                    } else {
                        *power + 1.
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
    pub weight: Weight,
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

        if let Some((fancy, subwave)) = self.embelish {
            let (x0, x1) = (
                offset
                    .quant_floor(subwave.period, subwave.offset)
                    .clamp(last.point.x, self.point.x),
                offset
                    .quant_ceil(subwave.period, subwave.offset)
                    .clamp(last.point.x, self.point.x),
            );

            let t = (offset - x0) / (x1 - x0);
            let odd_parity = ((offset - subwave.offset) / subwave.period).floor() as i32 % 2 != 0;

            let (dy0, dy1) =  match fancy {
                Fancy::Step => {(
                    dy * self.weight.eval(x0),
                    dy * self.weight.eval(x1)
                )},
                Fancy::Hop { alternate } => {
                    if alternate && odd_parity {(
                        dy * self.weight.eval(x0),
                        0.
                    )} else {(
                        0.,
                        dy * self.weight.eval(x1)
                    )}
                },
                Fancy::Oscilate { alternate } => {
                    let h0 = dy * self.weight.eval(x0);
                    let h1 = dy * self.weight.eval(x1);

                    if alternate && odd_parity {(
                        (dy - h0) * 0.5 + h0,
                        (dy - h1) * 0.5
                    )}
                    else {(
                        (dy - h0) * 0.5,
                        (dy - h1) * 0.5 + h1
                    )}
                },
            };

            last.point.y + dy0 + (dy1 - dy0) * subwave.weight.eval(t)
        } else {
            last.point.y + dy * self.weight.eval((offset - last.point.x) / (self.point.x - last.point.x))
        }
    }
}
