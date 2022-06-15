use derive_more::{Deref, DerefMut};
use noisy_float::prelude::*;

use macros::*;

use crate::automation::Weight;
use crate::utils::*;

#[derive(Default)]
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

struct SpannedBound<T> {
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

#[derive(Clone, Copy, Deref, DerefMut, Lerp)]
struct Scale(R32);
#[derive(Clone, Copy, Deref, DerefMut, Lerp)]
struct Rotation(R32);
#[derive(Clone, Copy, Deref, DerefMut, Lerp)]
struct Luminosity(T32);

#[derive(Clone, Copy, Deref, DerefMut)]
struct Rgba([T32; 4]);

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
    fn bound_sample() {
        let co_vals = [(0., 0.), (0.5, 0.), (1., 1.), (2., 1.), (3., 1.), (4., 1.)];

        co_vals
            .iter()
            .map(|&(input, output)| (r32(input), r32(output)))
            .for_each(|(input, output)| assert_eq!(bounds().interp_or_last(input), output));
    }
}
