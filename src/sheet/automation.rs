use bevy::prelude::*;
use noisy_float::prelude::*;
use tinyvec::TinyVec;

use super::{repeater::*, Modulation, Synth};
use crate::{hit::*, utils::*};

pub enum Weight {
    Constant,
    Quadratic(R32),
    Cubic(R32),
}

impl Weight {
    pub fn eval(&self, t: T32) -> T32 {
        let f = |x: f32, k: f32| x.signum() * x.abs().powf((k + k.signum()).abs().powf(k.signum()));

        match self {
            Weight::Constant => t32(1.),
            Weight::Quadratic(k) => t32(f(t.raw(), k.raw())),
            Weight::Cubic(k) => t32(((f(2. * t.raw() - 1., k.raw()) - 1.) / 2.) + 1.),
        }
    }
}

impl Default for Weight {
    fn default() -> Self {
        Self::Quadratic(r32(0.))
    }
}

#[derive(Default)]
pub struct Anchor<T> {
    x: P32,
    val: T,
    weight: Weight,
}

impl<T> Quantify for Anchor<T> {
    fn quantify(&self) -> P32 {
        self.x
    }
}

impl<T> Lerp for Anchor<T>
where
    T: Lerp,
{
    type Output = <T as Lerp>::Output;
    fn lerp(&self, next: &Self, t: T32) -> Self::Output {
        next.val.lerp(&self.val, next.weight.eval(t))
    }
}

#[derive(Deref, DerefMut, Component)]
pub struct Automation<T: Default>(pub TinyVec<[Anchor<T>; 6]>);

impl Synth for Automation<T32> {
    type Output = T32;

    fn play_from(&self, offset: P32, lower_clamp: T32, upper_clamp: T32) -> Self::Output {
        lower_clamp.lerp(
            &upper_clamp,
            self.interp(offset).unwrap_or_else(|anchor| anchor.val),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Weight::*;

    #[test]
    fn weight_inflections() {
        assert_eq!(Constant.eval(t32(0.)), t32(1.));
        assert_eq!(Constant.eval(t32(0.5)), t32(1.));
        assert_eq!(Constant.eval(t32(1.)), t32(1.));
        assert_eq!(Quadratic(r32(0.)).eval(t32(0.5)), t32(0.5));

        (-20..20).map(|i| i as f32).map(r32).for_each(|weight| {
            assert_eq!(Quadratic(weight).eval(t32(0.)), t32(0.));
            assert_eq!(Quadratic(weight).eval(t32(1.)), t32(1.));
            assert_eq!(Cubic(weight).eval(t32(0.)), t32(0.));
            assert_eq!(Cubic(weight).eval(t32(0.5)), t32(0.5));
            assert_eq!(Cubic(weight).eval(t32(1.)), t32(1.));
        })
    }

    #[test]
    #[rustfmt::skip]
    fn weight_symmetry() {
        (-20..=-1).chain(1..=20).map(|i| i as f32).map(r32).for_each(|weight| {
            assert_ne!(Quadratic(weight).eval(t32(0.5)), t32(0.5));
            assert_ne!(Cubic(weight).eval(t32(0.25)), t32(0.25));
            assert_ne!(Cubic(weight).eval(t32(0.75)), t32(0.75));

            (1..50).chain(51..100).map(|i| t32((i as f32) / 100.)).for_each(|t| {
                assert_eq!(Quadratic(weight).eval(t) - Quadratic(weight).eval(t), 0.);
                assert_eq!(Cubic(weight).eval(t) - Cubic(weight).eval(t), 0.);
            })
        })
    }

    #[test]
    fn weight_growth() {
        (-20..=20).map(|i| i as f32).map(r32).for_each(|weight| {
            (1..=100).map(|i| t32((i as f32) / 100.)).for_each(|t1| {
                let t0 = t1 - 0.01;
                assert!(Quadratic(weight).eval(t0) < Quadratic(weight).eval(t1));
                assert!(Cubic(weight).eval(t0) <= Cubic(weight).eval(t1));
            })
        })
    }
}
