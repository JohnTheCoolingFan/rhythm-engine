use bevy::prelude::*;
use derive_more::{Deref, DerefMut};
use noisy_float::prelude::*;

use macros::*;

use crate::automation::Weight;
use crate::utils::*;

#[derive(Default)]
pub struct Bound<T> {
    pub offset: R32,
    pub value: T,
}

impl<T> Quantify for Bound<T> {
    fn quantify(&self) -> R32 {
        self.offset
    }
}

impl<T> Sample for Bound<T>
where
    T: Sample,
{
    type Output = <T as Sample>::Output;
    fn sample(&self, other: &Self, t: T32) -> Self::Output {
        self.value.sample(&other.value, t)
    }
}

#[derive(Clone, Copy, Deref, DerefMut, Lerp, Sample)]
struct Scale(R32);
#[derive(Clone, Copy, Deref, DerefMut, Lerp, Sample)]
struct Rotation(R32);
#[derive(Clone, Copy, Deref, DerefMut, Lerp, Sample)]
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

struct TransColor {
    weight: Weight,
    rgba: Rgba,
}

impl Sample for TransColor {
    type Output = Rgba;
    fn sample(&self, other: &Self, t: T32) -> Self::Output {
        self.rgba.lerp(&other.rgba, other.weight.eval(t))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds() -> Vec<Bound<R32>> {
        vec![
            Bound {
                value: r32(0.),
                offset: r32(0.),
            },
            Bound {
                value: r32(1.),
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
            .for_each(|(input, output)| assert_eq!(bounds().sample(input), output));
    }
}
