use crate::utils::{math::*, seeker::*, misc::*};
use glam::Vec2;
use std::mem::swap;
use duplicate::duplicate;

#[derive(Debug, Copy, Clone)]
pub enum Weight {
    Constant {
        y_flip: bool
    },
    QuadLike{
        curvature: f32,
        x_flip: bool,
        y_flip: bool
    },
    CubeLike{
        curvature: f32,
        y_flip: bool
    },
}

impl Weight {
    pub fn cycle(&mut self) -> Self {
        let old = *self;
        *self = match self {
            Self::Constant{ .. } => Self::QuadLike{ curvature: 0., x_flip: false, y_flip: false },
            Self::QuadLike{ .. } => Self::CubeLike{ curvature: 0., y_flip: false },
            Self::CubeLike{ .. } => Self::Constant{ y_flip: false },
        };
        old
    }

    pub fn set_curvature(&mut self, new: f32) -> Result<f32, ()> {
        match self {
            Self::QuadLike{ref mut curvature, .. } | Self::CubeLike{ ref mut curvature, .. } => {
                let old = *curvature;
                *curvature = new.clamp(-30., 30.);
                Ok(old)
            }
            _ => Err(()),
        }
    }

    pub fn shift_curvature(&mut self, shift: f32) -> Result<f32, ()> {
        if let Self::QuadLike{ curvature, .. } | Self::CubeLike{ curvature, .. } = *self {
            self.set_curvature(shift + curvature)
        } else {
            Err(())
        }
    }

    pub fn flip_x(&mut self) -> Result<bool, ()> {
        match self {
            Self::QuadLike{ ref mut x_flip, .. } => {
                *x_flip = !*x_flip;
                Ok(!*x_flip)
            }
            _ => Err(())
        }
    }

    pub fn flip_y(&mut self) -> Self {
        let old = *self;
        match self {
            Self::Constant{ ref mut y_flip, .. }
            | Self::QuadLike{ ref mut y_flip, .. }
            | Self::CubeLike{ ref mut y_flip, .. }=> {
                *y_flip = !*y_flip;
            } 
        }
        old
    }

    #[rustfmt::skip]
    pub fn eval(&self, t: f32) -> f32 {
        let (curvature, x_flip, y_flip) = match self {
            Self::Constant{ y_flip } => return if *y_flip { 0. } else { 1. },
            Self::QuadLike{ curvature, x_flip, y_flip } => (*curvature, *x_flip, *y_flip),
            Self::CubeLike{ curvature, y_flip } => (*curvature, false, *y_flip)
        };

        //cubic is basically 2 quadratics with the 2nd
        //being inverted about the half way point
        let (starting, delta, mut x) = if let Self::CubeLike{ .. } = self {
            if 0.5 < t {
                (1.,    -0.5,   (0.5 - t % 0.5) / 0.5   )
            } else {
                (0.,    0.5,    t / 0.5                 )
            }
        } else {
                (0.,    1.,     t                       )
        };

        if x_flip { x = 1. - x }

        let out = starting + delta * x.powf(
            if curvature < 0. {
                1. / (curvature.abs() + 1.)
            } else {
                curvature + 1.
            }
        );

        if y_flip { 1. - out } else { out }
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
    pub(in super::super) point: Vec2,
    pub weight: Weight,
    pub subwave: SubWave,
}

impl Anchor {
    pub fn new(p: Vec2) -> Self {
        debug_assert!((0.0..=1.0).contains(&p.y), "anchor point y out of range");
        Self {
            point: p,
            weight: Weight::QuadLike{ curvature: 0., x_flip: false, y_flip: false },
            subwave: SubWave {
                offset: 0.,
                period: 0.,
                weight: Weight::QuadLike{ curvature: 0., x_flip: false, y_flip: false },
                mode: SubWaveMode::Off,
            },
        }
    }

    #[rustfmt::skip]
    pub fn eval(&self, last: &Self, offset: f32) -> f32 {
        let start = *last;
        let end = *self;

        let dy = end.point.y - start.point.y;
        let dx = end.point.x - start.point.x; 

        if let SubWaveMode::Off = end.subwave.mode {
            return start.point.y
                + dy 
                * end.weight.eval((offset - start.point.x) / dx).if_nan(0.)
        }
        
        let x0 = (offset - start.point.x).quant_floor(
            end.subwave.period,
            end.subwave.offset
        );

        let x1 = x0 + end.subwave.period;

        let odd_parity = 
            ((offset - end.subwave.offset - start.point.x) / end.subwave.period)
                .if_nan(0.)
                .floor()
                as i32 % 2 != 0;

        let (x_alt, y_alt) = match end.subwave.mode {
            SubWaveMode::Oscilate{ x_alt, y_alt } | SubWaveMode::Hop{ x_alt, y_alt } => (
                x_alt, y_alt
            ),
            _ => (false , false)
        };

        let (t0, t1) = ((x0 / dx).if_nan(0.), (x1 / dx).if_nan(0.));
        
        let (dy0, dy1) = match end.subwave.mode {
            SubWaveMode::Step => (
                dy * end.weight.eval(t0),
                dy * end.weight.eval(t1),
            ),
            SubWaveMode::Hop{ .. } => {
                if (x_alt ^ y_alt) && odd_parity {(
                    0.,
                    dy * end.weight.eval(t0),
                )}
                else {(
                    0.,
                    dy * end.weight.eval(t1),
                )}
            },
            SubWaveMode::Oscilate { .. } => {
                let h0 = dy * end.weight.eval(t0);
                let h1 = dy * end.weight.eval(t1);

                if (x_alt ^ y_alt) && odd_parity {(
                    (dy - h1) * 0.5,
                    (dy - h0) * 0.5 + h0
                )}
                else {(
                    (dy - h0) * 0.5,
                    (dy - h1) * 0.5 + h1
                )}
            },
            _ => unreachable!()
        };

        let l = {
            let t = ((offset - start.point.x - x0) / end.subwave.period).if_nan(0.);
            end.subwave.weight.eval(
                if x_alt && odd_parity { t.lerp_invert() } else { t }
            )
        };

        let (min, max) = if start.point.y < end.point.y  {
            (start.point.y, end.point.y)
        } else {
            (end.point.y, start.point.y)
        };

        (start.point.y + dy0 + 
            (dy1 - dy0) * if y_alt && odd_parity {
                l.lerp_invert()
            }
            else {
                l
            }
        ).clamp(min, max)
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

impl<'a> SeekerTypes for BPSeeker<'a, Anchor> {
    type Source = Anchor;
    type Output = f32;
}

impl<'a> Exhibit for BPSeeker<'a, Anchor> { 
    fn exhibit(&self, offset: f32) -> Self::Output {
        if self.over_run() {
            self.vec()[FromEnd(0)].point.y
        }
        else if self.under_run() {
            self.vec()[0].point.y
        }
        else {
            self.current().eval(self.previous(), offset)
        }
    }
}
