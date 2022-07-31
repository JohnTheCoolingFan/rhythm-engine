use bevy::prelude::*;
use derive_more::{Deref, DerefMut};
use noisy_float::prelude::*;

use macros::*;

use super::automation::*;
use crate::utils::*;

#[derive(Default, Clone, Copy, Deref, DerefMut, Lerp)]
pub struct Scale(R32);
#[derive(Default, Clone, Copy, Deref, DerefMut, Lerp)]
pub struct Rotation(R32);
#[derive(Component)]
pub struct GeometryCtrl(Vec2);

#[derive(Component, Default, Clone, Copy, Deref, DerefMut, Lerp)]
pub struct Luminosity(T32);
#[derive(Component, Default, Clone, Copy, Deref, DerefMut)]
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

#[derive(Component)]
pub struct BoundSequence<T: Default> {
    upper: Automation<T>,
    lower: Automation<T>,
}

/*impl<T: Default> BoundSequence<T>
where
    T: Default + Copy + Quantify + Lerp + Lerp<Output = T>,
{
    pub fn play(&self, offset: P32, t: T32) -> T {
        self.lower.play(offset).lerp(&self.upper.play(offset), t)
    }
}*/

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
