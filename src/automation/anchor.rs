use crate::utils::{math::*, seeker::*, misc::*};
use std::default::Default;
use glam::Vec2;

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
            Self::QuadLike{ ref mut curvature, .. } => {
                //"floating point types cannot be used in patterns" :(
                if *curvature == 0. {
                    Self::CubeLike{ curvature: 0., y_flip: false }
                }
                else {
                    *curvature = 0.;
                    *self
                }
            },
            Self::CubeLike{ curvature, .. } => {
                if *curvature == 0. {
                    Self::Constant{ y_flip: false }
                }
                else {
                    *curvature = 0.;
                    *self
                }
            }
        };
        old
    }

    pub fn curvature(&self) -> Option<&f32> {
        if let Self::QuadLike{ref curvature, .. } | Self::CubeLike{ ref curvature, .. } = &self {
            Some(curvature)
        }
        else {
            None
        }
    }

    pub fn set_curvature(&mut self, new: f32) -> Result<f32, ()> {
        if let Self::QuadLike{ref mut curvature, .. } | Self::CubeLike{ ref mut curvature, .. } = self {
            let old = *curvature;
            *curvature = new.clamp(-10., 10.);
            Ok(old)
        }
        else {
            Err(())
        }
    }

    pub fn shift_curvature(&mut self, shift: f32) -> Result<f32, ()> {
        if let Self::QuadLike{ref mut curvature, .. } | Self::CubeLike{ ref mut curvature, .. } = self {
            let old = *curvature;
            *curvature = (old + shift).clamp(-10., 10.);
            Ok(old)
        }
        else {
            Err(())
        }
    }


    pub fn x_flip(&self) -> Option<&bool> {
        if let Self::QuadLike{ ref x_flip, .. } = self {
            Some(x_flip)
        }
        else {
            None
        }
    }

    pub fn x_flip_mut(&mut self) -> Option<&mut bool> {
        if let Self::QuadLike{ ref mut x_flip, .. } = self {
            Some(x_flip)
        }
        else {
            None
        }
    }


    pub fn y_flip(&self) -> &bool {
        let (
            Self::QuadLike{ ref y_flip, .. } |
            Self::CubeLike{ ref y_flip, .. } |
            Self::Constant{ ref y_flip }
        ) = self;
        y_flip
    }

    pub fn y_flip_mut(&mut self) -> &mut bool {
        let (
            Self::QuadLike{ ref mut y_flip, .. } |
            Self::CubeLike{ ref mut y_flip, .. } |
            Self::Constant{ ref mut y_flip }
        ) = self;
        y_flip
    }


    #[rustfmt::skip]
    pub fn eval(&self, t: f32) -> f32 {
        let (curvature, x_flip, y_flip) = match self {
            Self::Constant{ y_flip } => return if *y_flip { 0. } else { 1. },
            Self::QuadLike{ curvature, x_flip, y_flip } => (*curvature, *x_flip, *y_flip),
            Self::CubeLike{ curvature, y_flip } => (*curvature, false, *y_flip)
        };
 
        let (starting, delta, mut x) = if let Self::CubeLike{ .. } = self {
            if 0.5 < t {
                (1.,    -0.5,   (0.5 - (t - 0.5)) / 0.5 )
            } else {
                (0.,    0.5,    t / 0.5                 )
            }
        } else {
                (0.,    1.,     t                       )
        };

        let k = curvature.abs() + 1.;

        if x_flip ^ (1.5 < k && curvature < 0.) { x = 1. - x }

        let y = if 1.5 < k {
            (k.powi(5).powf(x) - 1.) / (k.powi(5) - 1.)
        } else {
            x
        };

        {
            let out = starting + delta * if 1.5 < k && curvature < 0. { 1. - y } else { y };
            if y_flip { 1. - out } else { out }
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
    Hop,
    Oscilate,
}

impl SubWaveMode {
    pub fn cycle(&mut self) -> Self {
        let old = *self;
        *self = match self {
            Self::Off => Self::Step,
            Self::Step => Self::Hop,
            Self::Hop { .. } => Self::Oscilate,
            Self::Oscilate { .. } => Self::Off,
        };
        old
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
        let start = last;
        let mut end = *self;

        let dy = end.point.y - start.point.y;
        let dx = end.point.x - start.point.x; 
        
        if dx < end.subwave.period {
            return end.point.y
        }
        if matches!(end.subwave.mode, SubWaveMode::Off) || end.subwave.period == 0. {
            return start.point.y
                + dy 
                * end.weight.eval((offset - start.point.x) / dx)
        }

        let mut f = false;
        let (x_alt, y_alt) = match end.subwave.weight {
            Weight::Constant{ ref mut y_flip } | Weight::CubeLike{ ref mut y_flip, .. } => (&mut f, y_flip),
            Weight::QuadLike{ ref mut x_flip, ref mut y_flip, .. } => (x_flip, y_flip)
        };

        let odd_parity = 
            ((offset - end.subwave.offset - start.point.x) / end.subwave.period)
                .floor()
                .abs()
                as i32 % 2 == 1;

        if !odd_parity {
            *x_alt = false;
            *y_alt = false;
        }

        let x0 = (offset - start.point.x).quant_floor(
            end.subwave.period,
            end.subwave.offset
        );
 
        let e0 = end.weight.eval(x0 / dx);
        let e1 = end.weight.eval(x0 / dx + (end.subwave.period / dx));

        let (sub_start, sub_scale) = match end.subwave.mode {
            SubWaveMode::Step => {
                *x_alt = false;
                *y_alt = false;
                (e0 * dy, e1 - e0)
            },
            SubWaveMode::Hop => {
                if odd_parity && (*x_alt ^ *y_alt) {
                    (0., e0)
                }
                else {
                    (0., e1)
                }
            }
            SubWaveMode::Oscilate => {
                let h0 = dy * e0;
                let h1 = dy * e1;
                if (*x_alt ^ *y_alt) && odd_parity {
                    ((dy - h1) * 0.5, e0 + (e1 - e0) * 0.5)
                }
                else {
                    ((dy - h0) * 0.5, e0 + (e1 - e0) * 0.5)
                }            
            }
            _ => (0., 0.)
        };

        let (mut min, mut max) = (start.point.y, end.point.y);
        if max < min { std::mem::swap(&mut min, &mut max); }

        (start.point.y + sub_start + dy * sub_scale * end.subwave.weight.eval(
                (offset - start.point.x - x0) / end.subwave.period
            )
        )
        .clamp(min, max)
    }

}

impl Default for Anchor {
    fn default() -> Self {
        Self {
            point: Vec2::new(0., 0.),
            weight: Weight::Constant{ y_flip: false },
            subwave: SubWave {
                mode: SubWaveMode::Off,
                offset: 0.,
                weight: Weight::Constant{ y_flip: false },
                period: 0.
            }
        }
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

impl<'a> SeekerTypes for Seeker<&'a TVec<Anchor>, usize> {
    type Source = Anchor;
    type Output = f32;
}

impl<'a> Exhibit for Seeker<&'a TVec<Anchor>, usize> { 
    fn exhibit(&self, offset: f32) -> Self::Output {
        match (self.previous(), self.current()) {
            (Some(prev), Ok(curr)) => curr.eval(prev, offset),
            (None, Ok(curr) | Err(curr))
            | (_, Err(curr)) => curr.point.y,
        }
    }
}
