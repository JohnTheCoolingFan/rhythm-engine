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
            .lerp(&other.bound.scalar, other.weight.eval(t.inv()))
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
        t32(other.point.y).lerp(&t32(self.point.y), other.weight.eval(t))
    }
}

#[derive(Component)]
pub struct BoundSequence<T: Default> {
    upper_bounds: TinyVec<[T; 4]>,
    anchors: TinyVec<[Anchor; 8]>,
    lower_bounds: TinyVec<[T; 4]>,
    repeat_bounds: bool,
}

#[derive(Default, Component)]
pub struct BoundSequenceCache;

type BoundSequenceOutput<T> = <<T as Lerp>::Output as Lerp>::Output;

#[rustfmt::skip]
impl<T> AutomationClip for BoundSequence<T>
where
    T: Default + Quantify + Lerp,
    <T as Lerp>::Output: Lerp<Output = <T as Lerp>::Output>,
{
    type Output = Option<BoundSequenceOutput<T>>;
    type ClipCache = BoundSequenceCache;

    fn duration(&self) -> R32 {
        r32(self.anchors.last().unwrap().point.x)
    }

    fn play(
        &self,
        clip_time: R32,
        repeat_time: R32,
        lower_clamp: T32,
        upper_clamp: T32,
        _clip_cache: &mut Self::ClipCache
    )
        -> Self::Output
    {
        self.anchors.interp(repeat_time).ok().map(|lerp_amount| {
            let bound_time = if self.repeat_bounds { repeat_time } else { clip_time };

            let (Ok(lower) | Err(lower)) = self
                .lower_bounds
                .interp(bound_time)
                .map_err(|last| last.lerp(last, t32(0.)));

            let (Ok(upper) | Err(upper)) = self
                .upper_bounds
                .interp(bound_time)
                .map_err(|last| last.lerp(last, t32(0.)));

            lower.lerp(&upper, lower_clamp.lerp(&upper_clamp, lerp_amount))
        })
    }
}

/*#[cfg(test)]
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
}*/
