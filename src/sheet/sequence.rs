use std::marker::PhantomData;

use bevy::prelude::*;
use derive_more::{Deref, DerefMut};
use noisy_float::prelude::*;
use tap::Pipe;

use super::automation::*;
use crate::{sheet::spline::*, utils::*};

#[derive(Deref, DerefMut, Default, Component, Clone, Copy)]
pub struct Scalar<Marker, Type = R32> {
    #[deref]
    #[deref_mut]
    value: Type,
    _phantom: PhantomData<Marker>,
}

impl<Marker, Type: Lerp<Output = Type>> Lerp for Scalar<Marker, Type> {
    type Output = Self;
    fn lerp(&self, next: &Self, t: T32) -> Self::Output {
        Self {
            value: self.value.lerp(&next.value, t),
            _phantom: PhantomData,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct MarkerLuminosity;
#[derive(Default, Clone, Copy)]
pub struct MarkerRotation;
#[derive(Default, Clone, Copy)]
pub struct MarkerScale;

pub type Luminosity = Scalar<MarkerLuminosity, T32>;
pub type Rotation = Scalar<MarkerRotation>;
pub type Scale = Scalar<MarkerScale>;

#[derive(Deref, DerefMut, Default, Component, Clone, Copy)]
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

impl Sequence<Spline> {
    #[rustfmt::skip]
    pub fn play(&self, t: T32, offset: P32) -> Vec2 {
        match self.at_or_after(offset) {
            [prev, curr, ..] => offset
                .completion_ratio(prev.quantify(), curr.quantify())
                .pipe(|weight| prev.val.play(t).lerp(curr.val.play(t), weight.raw())),
            [single] => single.val.play(t),
            _ => panic!("Unexpected existing no item control table"),
        }
    }
}

#[derive(Deref, DerefMut, Component)]
pub struct Sequence<T: Default>(Automation<T>);

impl<T: Default + Clone + Copy + Lerp<Output = T>> Sequence<T> {
    pub fn play(&self, offset: P32) -> <T as Lerp>::Output {
        self.interp(offset).unwrap_or_else(|anchor| anchor.val)
    }
}

#[derive(Deref, DerefMut, Component)]
pub struct PrimarySequence<T>(T);

#[derive(Deref, DerefMut, Component)]
pub struct SecondarySequence<T>(T);

#[derive(Deref, DerefMut, Component)]
pub struct GeometryCtrl(Vec2);
