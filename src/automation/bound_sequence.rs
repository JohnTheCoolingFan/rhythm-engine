use bevy::prelude::*;
use derive_more::{Deref, DerefMut};
use noisy_float::prelude::*;
use tinyvec::TinyVec;

use macros::*;

use crate::{automation::*, utils::*};

#[derive(Default, Component)]
pub struct ScalarBound<T> {
    pub offset: R32,
    pub scalar: T,
}

impl<T> Quantify for ScalarBound<T> {
    fn quantify(&self) -> R32 {
        self.offset
    }
}

impl<T> Lerp for ScalarBound<T>
where
    T: Copy + Lerp<Output = T>,
{
    type Output = <T as Lerp>::Output;
    fn lerp(&self, other: &Self, _t: T32) -> Self::Output {
        other.scalar
    }
}

#[derive(Default, Component)]
pub struct SpannedBound<T> {
    weight: Weight,
    bound: ScalarBound<T>,
}

impl<T> Lerp for SpannedBound<T>
where
    T: Copy + Lerp<Output = T>,
{
    type Output = <T as Lerp>::Output;
    fn lerp(&self, other: &Self, t: T32) -> Self::Output {
        self.bound
            .scalar
            .lerp(&other.bound.scalar, self.weight.eval(t.inv()))
    }
}

#[derive(Default, Clone, Copy, Deref, DerefMut, Lerp)]
pub struct Scale(R32);
#[derive(Default, Clone, Copy, Deref, DerefMut, Lerp)]
pub struct Rotation(R32);
#[derive(Default, Clone, Copy, Deref, DerefMut, Lerp)]
pub struct Luminosity(T32);

#[derive(Default, Clone, Copy, Deref, DerefMut)]
pub struct Rgba([T32; 4]);

impl Lerp for Rgba {
    type Output = Self;
    fn lerp(&self, other: &Self, t: T32) -> Self::Output {
        let mut iter = self
            .iter()
            .zip(other.iter())
            .map(|(from, to)| from.lerp(to, t));

        Rgba([(); 4].map(|_| iter.next().unwrap()))
    }
}

#[derive(Default)]
struct Anchor {
    point: Vec2,
    weight: Weight,
}

impl Quantify for Anchor {
    fn quantify(&self) -> R32 {
        r32(self.point.x)
    }
}

impl Lerp for Anchor {
    type Output = T32;
    fn lerp(&self, other: &Self, t: T32) -> Self::Output {
        t32(other.point.y).lerp(&t32(self.point.y), self.weight.eval(t))
    }
}

struct RepeaterClamp {
    start: T32,
    end: T32,
    weight: Weight,
}

impl RepeaterClamp {
    pub fn eval(&self, t: T32) -> T32 {
        self.start.lerp(&self.end, self.weight.eval(t))
    }
}

#[derive(Component)]
struct Repeater {
    duration: R32,
    repeat_bounds: bool,
    ceil: RepeaterClamp,
    floor: RepeaterClamp,
}

#[derive(Component)]
pub struct BoundSequence<T: Default> {
    upper_bounds: TinyVec<[T; 4]>,
    anchors: TinyVec<[Anchor; 8]>,
    lower_bounds: TinyVec<[T; 4]>,
    repeater: Option<Repeater>,
}

type BoundSequenceOutput<T> = <<T as Lerp>::Output as Lerp>::Output;

#[rustfmt::skip]
impl<T> AutomationClip for BoundSequence<T>
where
    T: Default + Quantify + Lerp,
    <T as Lerp>::Output: Lerp<Output = <T as Lerp>::Output>,
{
    type Output = Option<BoundSequenceOutput<T>>;

    fn play(&self, offset: R32) -> Self::Output {
        self.repeater
            .as_ref()
            .and_then(|Repeater { duration, floor, ceil, repeat_bounds }| {
                (offset < *duration).then(|| {
                    let period = r32(self.anchors.last().unwrap().point.x);
                    let repeater_offset = offset % period;

                    self.anchors.interp(repeater_offset).map(|lerp_amount| {
                        let clamp_offset = (offset / period)
                            .trunc()
                            .unit_interval(r32(0.), *duration);
                        let lerp_amount = floor
                            .eval(clamp_offset)
                            .lerp(&ceil.eval(clamp_offset), lerp_amount);

                        (if *repeat_bounds { repeater_offset } else { offset }, lerp_amount)
                    })
                })
            })
            .unwrap_or_else(|| self
                .anchors
                .interp(offset)
                .map(|lerp_amount| (offset, lerp_amount))
            )
            .map(|(bound_offset, lerp_amount)| self
                .lower_bounds
                .interp_or_last(bound_offset)
                .lerp(&self.upper_bounds.interp_or_last(bound_offset), lerp_amount)
            )
    }
}

//impl<T: Default> AutomationClip for BoundSequence<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds() -> Vec<ScalarBound<R32>> {
        vec![
            ScalarBound {
                scalar: r32(0.),
                offset: r32(0.),
            },
            ScalarBound {
                scalar: r32(1.),
                offset: r32(1.),
            },
        ]
    }

    #[test]
    fn scalar_bound_sample() {
        let co_vals = [(0., 0.), (0.5, 0.), (1., 1.), (2., 1.), (3., 1.), (4., 1.)];

        co_vals
            .iter()
            .map(|&(input, output)| (r32(input), r32(output)))
            .for_each(|(input, output)| assert_eq!(bounds().interp_or_last(input), output));
    }
}
