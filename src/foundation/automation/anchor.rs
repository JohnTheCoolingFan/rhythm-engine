use crate::utils::{math::*, seeker::*};
use glam::Vec2;

#[derive(Debug, Copy, Clone)]
pub enum Weight {
    ForwardBias,
    QuadLike(f32),
    CubeLike(f32),
    ReverseBias,
}

impl Weight {
    pub fn cycle(&mut self) -> Self {
        let old = *self;
        *self = match self {
            Self::ForwardBias => Self::QuadLike(0.),
            Self::QuadLike(_) => Self::CubeLike(0.),
            Self::CubeLike(_) => Self::ReverseBias,
            Self::ReverseBias => Self::ForwardBias,
        };
        old
    }

    pub fn set_power(&mut self, new: f32) -> Result<f32, ()> {
        match self {
            Self::QuadLike(ref power) | Self::CubeLike(ref power) => {
                let old = *power;
                *power = new.clamp(-30., 30.);
                Ok(old)
            }
            _ => Err(()),
        }
    }

    pub fn shift_power(&mut self, shift: f32) -> Result<f32, ()> {
        if let Self::QuadLike(power) | Self::CubeLike(power) = self {
            self.set_power(shift + *power)
        } else {
            Err(())
        }
    }

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
//
//
//
//
//
#[derive(Debug, Clone, Copy)]
pub enum SubWaveMode {
    Off,
    Step,
    Hop { alternate: bool },
    Oscilate { alternate: bool },
}

impl SubWaveMode {
    pub fn cycle(&mut self) -> Self {
        let old = *self;
        *self = match self {
            Self::Off => Self::Step,
            Self::Step => Self::Hop { alternate: false },
            Self::Hop { .. } => Self::Oscilate { alternate: false },
            Self::Oscilate { .. } => Self::Off,
        };
        old
    }

    pub fn toggle_alternate(&mut self) -> Result<(), ()> {
        match self {
            Self::Hop { ref mut alternate } | Self::Oscilate { ref mut alternate } => {
                *alternate = !*alternate;
                Ok(())
            }
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SubWave {
    pub(super) period: f32,
    pub offset: f32,
    pub weight: Weight,
    pub mode: SubWaveMode,
}

impl SubWave {
    pub fn period(&self) -> &f32 {
        &self.period
    }

    pub fn set_period(&mut self, val: f32) -> f32 {
        let old = self.period;
        self.period = val.clamp(0., f32::MAX);
        old
    }

    pub fn shift_period(&mut self, val: f32) -> f32 {
        self.set_period(val + self.period)
    }
}
//
//
//
//
//
#[derive(Debug, Clone, Copy)]
pub struct Anchor {
    pub(super) point: Vec2,
    pub weight: Weight,
    pub subwave: SubWave,
}

impl Anchor {
    pub fn new(p: Vec2) -> Self {
        debug_assert!(0. <= p.y && p.y <= 1., "anchor point y out of range");
        Self {
            point: p,
            weight: Weight::QuadLike(0.),
            subwave: SubWave {
                offset: 0.,
                period: 0.,
                weight: Weight::ForwardBias,
                mode: SubWaveMode::Off,
            },
        }
    }

    pub fn point(&self) -> &Vec2 {
        &self.point
    }

    #[rustfmt::skip]
    pub fn interp(&self, last: &Self, offset: f32) -> f32 {
        //must take last point and raw offset instead of t
        //otherwise quantizing would have to be done by caller
        debug_assert!(last.point.x <= self.point.x, "self < last");
        debug_assert!(
            last.point.x <= offset && offset <= self.point.x,
            "offset out of bounds"
        );

        let dy = self.point.y - last.point.y; 

        if let SubWaveMode::Off = self.subwave.mode {
            last.point.y + dy * self.weight.eval((offset - last.point.x) / (self.point.x - last.point.x))
        }
        else {
            let (x0, x1) = (
                (last.point.x 
                    + (offset - last.point.x)
                        .quant_floor(self.subwave.period, self.subwave.offset))
                .clamp(last.point.x, self.point.x),
                
                (
                    last.point.x 
                    + (offset - last.point.y).quant_ceil(self.subwave.period, self.subwave.offset)
                ).clamp(last.point.x, self.point.x),
            );

            let t = (offset - x0) / (x1 - x0);
            let odd_parity = ((offset - self.subwave.offset) / self.subwave.period).floor() as i32 % 2 != 0;

            let (dy0, dy1) =  match self.subwave.mode {
                SubWaveMode::Step => {(
                    dy * self.weight.eval(x0),
                    dy * self.weight.eval(x1)
                )},
                SubWaveMode::Hop { alternate } => {
                    if alternate && odd_parity {(
                        dy * self.weight.eval(x0),
                        0.
                    )} else {(
                        0.,
                        dy * self.weight.eval(x1)
                    )}
                },
                SubWaveMode::Oscilate { alternate } => {
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
                _ => unreachable!()
            };

            last.point.y + dy0 + (dy1 - dy0) * self.subwave.weight.eval(t)
        }
    }
}

impl Seekable for Anchor {
    type Output = f32;
    type Quantifier = f32;

    fn quantify(&self) -> f32 {
        self.point.x
    }

    fn exhibit(&self, t: f32, seeker: &Seeker<Self>) -> Self {
        if seeker.over_run() | self.under_run() {
            self.y
        }
        else {
            self.interp(
                seeker.vec()[seeker.index() - 1],
                t
            )
        }
    }
}
