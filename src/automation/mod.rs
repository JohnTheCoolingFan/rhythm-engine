use crate::timing::*;
use crate::utils::*;
use bevy::prelude::*;
use noisy_float::prelude::*;
use tap::Pipe;
use tinyvec::*;

pub mod sequence;
pub mod spline;

#[derive(Clone)]
pub enum Weight {
    Constant,
    Quadratic(R32),
    Cubic(R32),
}

impl Weight {
    #[rustfmt::skip]
    pub fn eval(&self, t: T32) -> T32 {
        let func = |x: f32, k: f32| (k + k.signum())
            .abs()
            .powf(k.signum())
            .pipe(|power| x.signum() * x.abs().powf(power));

        match self {
            Weight::Constant => t32(1.),
            Weight::Quadratic(k) => t32(func(t.raw(), k.raw())),
            Weight::Cubic(k) => (2. * t.raw() - 1.)
                .pipe(|x| func(x, k.raw()))
                .pipe(|output| (output - 1.) / 2. + 1.)
                .pipe(t32),
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

#[derive(Default, Deref, DerefMut, Component)]
pub struct Automation<T: Default>(pub Vec<Anchor<T>>);

impl Automation<T32> {
    #[rustfmt::skip]
    pub fn play(&self, ClampedTime { offset, lower_clamp, upper_clamp }: ClampedTime) -> T32 {
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

    #[test]
    #[rustfmt::skip]
    fn play_automation() {
        let automation = Automation(vec![
            Anchor { x: p32(0.0), val: t32(0.), weight:  Constant },
            Anchor { x: p32(1.0), val: t32(1.0), weight: Quadratic(r32(0.)) },
            Anchor { x: p32(2.0), val: t32(0.5), weight: Constant },
            Anchor { x: p32(3.0), val: t32(0.0), weight: Quadratic(r32(0.)) }
        ]);

        assert_eq!(automation.play(ClampedTime::new(p32(0.0))), 0.0);
        assert_eq!(automation.play(ClampedTime::new(p32(0.5))), 0.5);
        assert_eq!(automation.play(ClampedTime::new(p32(1.0))), 1.0);
        assert_eq!(automation.play(ClampedTime::new(p32(1.5))), 0.5);
        assert_eq!(automation.play(ClampedTime::new(p32(2.5))), 0.25);
        assert_eq!(automation.play(ClampedTime::new(p32(3.5))), 0.0);
    }
}
