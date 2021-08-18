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

    pub fn curvature(&self) -> Option<&f32> {
        if let Self::QuadLike{ref curvature, .. } | Self::CubeLike{ ref curvature, .. } = &self {
            Some(curvature)
        }
        else {
            None
        }
    }

    pub fn curvature_mut(&mut self) -> Option<&mut f32> {
        if let Self::QuadLike{ref mut curvature, .. } | Self::CubeLike{ ref mut curvature, .. } = self {
            Some(curvature)
        }
        else {
            None
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
        let start = last;
        let end = *self;

        let dy = end.point.y - start.point.y;
        let dx = end.point.x - start.point.x; 
        
        if dx.abs() < f32::EPSILON {
            return end.point.y
        }
        if matches!(end.subwave.mode, SubWaveMode::Off) || end.subwave.period == 0. {
            return start.point.y
                + dy 
                * end.weight.eval((offset - start.point.x) / dx)
        }

        let (ref mut x_alt, ref mut y_alt) = match end.subwave.weight {
            Weight::Constant{ y_flip } | Weight::CubeLike{ y_flip, .. } => (false, y_flip),
            Weight::QuadLike{ x_flip, y_flip, .. } => (x_flip, y_flip)
        };

        let odd_parity = 
            ((offset - end.subwave.offset - start.point.x) / end.subwave.period)
                .floor()
                as i32 % 2 == 1;

        if !odd_parity {
            *x_alt = false;
            *y_alt = false;
        }

        let x0 = (offset - start.point.x).quant_floor(
            end.subwave.period,
            end.subwave.offset
        );
        let x1 = x0 + start.subwave.period;

        let t0 = x0 / dx;
        let t1 = t0 + (end.subwave.period / dx);

        let (dy0, dy1) = match end.subwave.mode {
            SubWaveMode::Step => {
                *x_alt = false;
                (dy * end.weight.eval(t0), dy * end.weight.eval(t1))
            },
            _ => (0., 0.)
        };

        let o = start.point.y + dy0 + (dy1 - dy0) * end.subwave.weight.eval(
            (offset - start.point.x - x0) / end.subwave.period
        );

        if x0 < 0. { println!("x0: {}, eval out: {}", x0, o); }
        o
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
