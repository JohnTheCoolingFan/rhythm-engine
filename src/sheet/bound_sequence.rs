use bevy::prelude::*;
use derive_more::{Deref, DerefMut};
use noisy_float::prelude::*;

use macros::*;

use super::{
    automation::*,
    sheet::{Modulation, Synth},
};
use crate::utils::*;

#[derive(Default, Clone, Copy, Deref, DerefMut, Lerp)]
pub struct Scale(R32);
#[derive(Default, Clone, Copy, Deref, DerefMut, Lerp)]
pub struct Rotation(R32);
#[derive(Component, Deref)]
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

impl<T> Synth for BoundSequence<T>
where
    T: Default + Copy + Lerp + Lerp<Output = T>,
{
    type Output = (T, T);

    fn play(&self, offset: P32, _: T32, _: T32) -> Self::Output {
        (
            self.lower
                .interp(offset)
                .unwrap_or_else(|anchor| anchor.val),
            self.upper
                .interp(offset)
                .unwrap_or_else(|anchor| anchor.val),
        )
    }
}
