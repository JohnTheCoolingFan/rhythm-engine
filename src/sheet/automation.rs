use bevy::prelude::*;
use noisy_float::prelude::*;
use tap::Pipe;
use tinyvec::*;

use super::repeater::*;
use crate::utils::*;

#[derive(Clone)]
pub enum Weight {
    Constant,
    Quadratic(R64),
    Cubic(R64),
}

impl Weight {
    #[rustfmt::skip]
    pub fn eval(&self, t: T64) -> T64 {
        let func = |x: f64, k: f64| (k + k.signum())
            .abs()
            .powf(k.signum())
            .pipe(|power| x.signum() * x.abs().powf(power));

        match self {
            Weight::Constant => t64(1.),
            Weight::Quadratic(k) => t64(func(t.raw(), k.raw())),
            Weight::Cubic(k) => (2. * t.raw() - 1.)
                .pipe(|x| func(x, k.raw()))
                .pipe(|output| (output - 1.) / 2. + 1.)
                .pipe(t64),
        }
    }
}

impl Default for Weight {
    fn default() -> Self {
        Self::Quadratic(r64(0.))
    }
}

#[derive(Default)]
pub struct Anchor<T> {
    pub x: P64,
    pub val: T,
    pub weight: Weight,
}

impl<T> Quantify for Anchor<T> {
    fn quantify(&self) -> P64 {
        self.x
    }
}

impl<T> Lerp for Anchor<T>
where
    T: Lerp,
{
    type Output = <T as Lerp>::Output;

    fn lerp(&self, next: &Self, t: T64) -> Self::Output {
        self.val.lerp(&next.val, next.weight.eval(t))
    }
}

#[derive(Default, Deref, DerefMut, Component)]
pub struct Automation<T: Default>(pub TinyVec<[Anchor<T>; 6]>);

impl Automation<T64> {
    #[rustfmt::skip]
    pub fn play(&self, ClampedTime { offset, lower_clamp, upper_clamp }: ClampedTime) -> T64 {
        self.interp(offset)
            .unwrap_or_else(|anchor| anchor.val)
            .pipe(|t| lower_clamp.lerp(&upper_clamp, t))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};
    use Weight::*;

    #[test]
    fn weight_inflections() {
        assert_eq!(Constant.eval(t64(0.)), t64(1.));
        assert_eq!(Constant.eval(t64(0.5)), t64(1.));
        assert_eq!(Constant.eval(t64(1.)), t64(1.));
        assert_eq!(Quadratic(r64(0.)).eval(t64(0.5)), t64(0.5));

        (-20..20).map(|i| i as f64).map(r64).for_each(|weight| {
            assert_eq!(Quadratic(weight).eval(t64(0.)), t64(0.));
            assert_eq!(Quadratic(weight).eval(t64(1.)), t64(1.));
            assert_eq!(Cubic(weight).eval(t64(0.)), t64(0.));
            assert_eq!(Cubic(weight).eval(t64(0.5)), t64(0.5));
            assert_eq!(Cubic(weight).eval(t64(1.)), t64(1.));
        })
    }

    #[test]
    #[rustfmt::skip]
    fn weight_symmetry() {
        (-20..=-1).chain(1..=20).map(|i| i as f64).map(r64).for_each(|weight| {
            assert_ne!(Quadratic(weight).eval(t64(0.5)), t64(0.5));
            assert_ne!(Cubic(weight).eval(t64(0.25)), t64(0.25));
            assert_ne!(Cubic(weight).eval(t64(0.75)), t64(0.75));

            (1..50).chain(51..100).map(|i| t64((i as f64) / 100.)).for_each(|t| {
                assert_eq!(Quadratic(weight).eval(t) - Quadratic(weight).eval(t), 0.);
                assert_eq!(Cubic(weight).eval(t) - Cubic(weight).eval(t), 0.);
            })
        })
    }

    #[test]
    fn weight_growth() {
        (-20..=20).map(|i| i as f64).map(r64).for_each(|weight| {
            (1..=100).map(|i| t64((i as f64) / 100.)).for_each(|t1| {
                let t0 = t1 - 0.01;
                assert!(Quadratic(weight).eval(t0) < Quadratic(weight).eval(t1));
                assert!(Cubic(weight).eval(t0) <= Cubic(weight).eval(t1));
            })
        })
    }

    #[test]
    #[rustfmt::skip]
    fn play_automation() {
        let automation = Automation(tiny_vec![
            Anchor { x: p64(0.0), val: t64(0.), weight:  Constant },
            Anchor { x: p64(1.0), val: t64(1.0), weight: Quadratic(r64(0.)) },
            Anchor { x: p64(2.0), val: t64(0.5), weight: Constant },
            Anchor { x: p64(3.0), val: t64(0.0), weight: Quadratic(r64(0.)) }
        ]);

        assert_eq!(automation.play(ClampedTime::new(p64(0.0))), 0.0);
        assert_eq!(automation.play(ClampedTime::new(p64(0.5))), 0.5);
        assert_eq!(automation.play(ClampedTime::new(p64(1.0))), 1.0);
        assert_eq!(automation.play(ClampedTime::new(p64(1.5))), 0.5);
        assert_eq!(automation.play(ClampedTime::new(p64(2.5))), 0.25);
        assert_eq!(automation.play(ClampedTime::new(p64(3.5))), 0.0);
    }
}
