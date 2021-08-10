use crate::utils::{math::*, seeker::*, misc::*};
use glam::Vec2;
use duplicate::duplicate;

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
            Self::QuadLike(ref mut power) | Self::CubeLike(ref mut power) => {
                let old = *power;
                *power = new.clamp(-30., 30.);
                Ok(old)
            }
            _ => Err(()),
        }
    }

    pub fn shift_power(&mut self, shift: f32) -> Result<f32, ()> {
        if let Self::QuadLike(power) | Self::CubeLike(power) = *self {
            self.set_power(shift + power)
        } else {
            Err(())
        }
    }

    #[rustfmt::skip]
    pub fn eval(&self, t: f32) -> f32 {
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

    pub fn checked_eval(&self, t: f32) -> f32 {
        debug_assert!((0.0..=1.0).contains(&t), "t out of bounds");
        self.eval(t)
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
    Hop { x_alt: bool, y_alt: bool },
    Oscilate { x_alt: bool, y_alt: bool },
}

impl SubWaveMode {
    pub fn cycle(&mut self) -> Self {
        let old = *self;
        *self = match self {
            Self::Off => Self::Step,
            Self::Step => Self::Hop { x_alt: false, y_alt: false },
            Self::Hop { .. } => Self::Oscilate { x_alt: false, y_alt: false },
            Self::Oscilate { .. } => Self::Off,
        };
        old
    }


    #[duplicate(
        method                  axis_alt;
        [toggle_x_alternate]    [x_alt];
        [toggle_y_alternate]    [y_alt]
    )]
    pub fn method(&mut self) -> Result<(), ()> {
        match self {
            Self::Hop { ref mut axis_alt, .. } | Self::Oscilate { ref mut axis_alt, .. } => {
                *axis_alt = !*axis_alt;
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
        debug_assert!((0.0..=1.0).contains(&p.y), "anchor point y out of range");
        Self {
            point: p,
            weight: Weight::QuadLike(0.),
            subwave: SubWave {
                offset: 0.,
                period: 0.,
                weight: Weight::QuadLike(1.),
                mode: SubWaveMode::Off,
            },
        }
    }

    pub fn point(&self) -> &Vec2 {
        &self.point
    }
}
//
//
//
//
//
impl Quantify for Anchor {
    type Quantifier = f32;

    fn quantify(&self) -> f32 {
        self.point.x
    }
}

impl<'a> SeekerTypes for Seeker<&'a Vec<Anchor>, usize> {
    type Source = Anchor;
    type Output = f32;
}

impl<'a> Exhibit for Seeker<&'a Vec<Anchor>, usize> { 
    #[rustfmt::skip]
    fn exhibit(&self, offset: f32) -> f32 {
        if self.over_run() {
            self.vec()[FromEnd(0)].point.y
        }
        else if self.under_run() {
            self.vec()[0].point.y
        }
        else {
            let end = self.vec()[self.index()];
            let start = self.vec()[self.index() - 1];

            let dy = end.point.y - start.point.y;
            let dx = end.point.x - start.point.x; 

            if let SubWaveMode::Off = end.subwave.mode {
                start.point.y
                    + dy 
                    * end.weight.eval(
                        ((offset - start.point.x) / dx).if_nan(0.)
                    )
            } else {
                let (x0, x1) = (
                    start.point.x
                        + (offset - start.point.x).quant_floor(
                            end.subwave.period,
                            end.subwave.offset
                        ),
                    start.point.x
                        + (offset - start.point.x).quant_ceil(
                            end.subwave.period,
                            end.subwave.offset
                        )
                );

                let (x_alt, y_alt) = match end.subwave.mode {
                    SubWaveMode::Hop { x_alt, y_alt } | SubWaveMode::Oscilate { x_alt, y_alt } => {
                        (x_alt, y_alt)
                    },
                    _ => (false, false)
                };
 
                let t = ((offset - x0) / end.subwave.period).if_nan(0.);
                let (t0, t1) = ((x0 / dx).if_nan(0.), (x1 / dx).if_nan(0.));

                let odd_parity = (
                    (offset - end.subwave.offset - start.point.x) / end.subwave.period
                ).if_nan(0.).floor() as i32 % 2 != 0;

                let (y0, y1) = match end.subwave.mode {
                    SubWaveMode::Step => (
                        dy * end.weight.eval(t0),
                        dy * end.weight.eval(t0),
                    ),
                    SubWaveMode::Hop{ .. } => {
                        if x_alt && odd_parity {(
                            dy * end.weight.eval(t0),
                            dy * 0.
                        )}
                        else {(
                            dy * 0.,
                            dy * end.weight.eval(t0),
                        )}
                    },
                    SubWaveMode::Oscilate { .. } => {
                        let h0 = dy * end.weight.eval(t0);
                        let h1 = dy * end.weight.eval(t1);

                        if x_alt && odd_parity {(
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

                let w = end.subwave.weight.eval( if x_alt && odd_parity { t.lerp_invert() } else { t });

                start.point.y + y0 + (y1 - y0) * if y_alt && odd_parity { w.lerp_invert() } else { w }
            }
        }
    }
}
