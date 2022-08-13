use bevy::prelude::*;
use noisy_float::prelude::*;
use tinyvec::*;

use super::{repeater::*, Modulation, Synth};
use crate::{hit::*, utils::*};

#[derive(Clone)]
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
    pub x: P32,
    pub val: T,
    pub weight: Weight,
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
        self.val.lerp(&next.val, next.weight.eval(t))
    }
}

#[derive(Deref, DerefMut, Component)]
pub struct Automation<T: Default>(pub TinyVec<[Anchor<T>; 6]>);

impl Synth for Automation<T32> {
    type Output = T32;

    fn play(&self, offset: P32, lower_clamp: T32, upper_clamp: T32) -> Self::Output {
        lower_clamp.lerp(
            &upper_clamp,
            self.interp(offset).unwrap_or_else(|anchor| anchor.val),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};
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

    #[rustfmt::skip]
    #[test]
    fn play_automation() {
        let automation = Automation(tiny_vec![
            Anchor { x: p32(0.0), val: t32(0.), weight:  Constant },
            Anchor { x: p32(1.0), val: t32(1.0), weight: Quadratic(r32(0.)) },
            Anchor { x: p32(2.0), val: t32(0.5), weight: Constant },
            Anchor { x: p32(3.0), val: t32(0.0), weight: Quadratic(r32(0.)) }
        ]);

        assert_eq!(automation.play(p32(0.0), t32(0.), t32(1.)), 0.0);
        assert_eq!(automation.play(p32(0.5), t32(0.), t32(1.)), 0.5);
        assert_eq!(automation.play(p32(1.0), t32(0.), t32(1.)), 1.0);
        assert_eq!(automation.play(p32(1.5), t32(0.), t32(1.)), 0.5);
        assert_eq!(automation.play(p32(2.5), t32(0.), t32(1.)), 0.25);
        assert_eq!(automation.play(p32(3.5), t32(0.), t32(1.)), 0.0);
    }
}
