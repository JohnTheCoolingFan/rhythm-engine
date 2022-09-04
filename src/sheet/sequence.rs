use std::marker::PhantomData;

use bevy::prelude::*;
use derive_more::{Deref, DerefMut};
use noisy_float::prelude::*;
use tap::Pipe;

use macros::*;

use super::automation::*;
use crate::utils::*;

struct GeometryCtrl(Vec2);

#[derive(Default, Clone, Copy, Deref, DerefMut)]
pub struct Scalar<Marker, Type = R32> {
    #[deref]
    #[deref_mut]
    value: Type,
    _phantom: PhantomData<Marker>,
}

pub struct MarkerLuminosity;
pub struct MarkerRotation;
pub struct MarkerScale;

pub type Luminosity = Scalar<MarkerLuminosity, T32>;
pub type Rotation = Scalar<MarkerRotation>;
pub type Scale = Scalar<MarkerScale>;

impl<Marker, Type: Lerp<Output = Type>> Lerp for Scalar<Marker, Type> {
    type Output = Self;
    fn lerp(&self, next: &Self, t: T32) -> Self::Output {
        Self {
            value: self.value.lerp(&next.value, t),
            _phantom: PhantomData,
        }
    }
}

#[derive(Default, Clone, Copy, Deref, DerefMut)]
pub struct Rgba([T32; 4]);

impl Lerp for Rgba {
    type Output = Self;

    fn lerp(&self, other: &Self, t: T32) -> Self::Output {
        self.iter()
            .zip(other.iter())
            .map(|(from, to)| from.lerp(to, t))
            .pipe_ref_mut(|iter| [(); 4].map(|_| iter.next().unwrap()))
            .pipe(Rgba)
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct Sequence<T: Default>(Automation<T>);

#[derive(Component)]
struct TopBound;

#[derive(Component)]
struct BottomBound;
