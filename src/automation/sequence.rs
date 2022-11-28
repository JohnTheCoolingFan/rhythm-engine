use std::marker::PhantomData;

use bevy::{math::DVec2, prelude::*};
use derive_more::{Deref, DerefMut};
use noisy_float::prelude::*;
use tap::Pipe;

use super::{spline::*, *};

#[derive(Deref, DerefMut, Default, Component, Clone, Copy)]
pub struct Scalar<Marker, Type = R64> {
    #[deref]
    #[deref_mut]
    value: Type,
    _phantom: PhantomData<Marker>,
}

impl<Marker, Type: Lerp<Output = Type>> Lerp for Scalar<Marker, Type> {
    type Output = Self;
    fn lerp(&self, next: &Self, t: T64) -> Self::Output {
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

pub type Luminosity = Scalar<MarkerLuminosity, T64>;
pub type Rotation = Scalar<MarkerRotation>;
pub type Scale = Scalar<MarkerScale>;

#[derive(Deref, DerefMut, Default, Component, Clone, Copy)]
pub struct Rgba([T64; 4]);

impl Lerp for Rgba {
    type Output = Self;

    fn lerp(&self, other: &Self, t: T64) -> Self::Output {
        self.iter()
            .zip(other.iter())
            .map(|(from, to)| from.lerp(to, t))
            .pipe_ref_mut(|iter| [(); 4].map(|_| iter.next().unwrap()))
            .pipe(Rgba)
    }
}

impl Sequence<Spline> {
    #[rustfmt::skip]
    pub fn play(&self, t: T64, offset: P64) -> DVec2 {
        match self.at_or_after(offset) {
            [prev, curr, ..] => offset
                .completion_ratio(prev.quantify(), curr.quantify())
                .pipe(|weight| prev.val.play(t).lerp(curr.val.play(t), weight.raw())),
            [single] => single.val.play(t),
            _ => panic!("Unexpected existing no item control table"),
        }
    }
}

#[derive(Default, Deref, DerefMut, Component)]
pub struct Sequence<T: Default>(Automation<T>);

impl<T: Default + Clone + Copy + Lerp<Output = T>> Sequence<T> {
    pub fn play(&self, offset: P64) -> <T as Lerp>::Output {
        self.interp(offset).unwrap_or_else(|anchor| anchor.val)
    }
}

// Sequences can either be simple sequences in which case they are enough to produce modulations.
// Or the can be composed of 2 sequences. They then require an Automation to produce modulations.
#[derive(Deref, DerefMut, Component)]
pub struct PrimarySequence<T>(T);

#[derive(Deref, DerefMut, Component)]
pub struct SecondarySequence<T>(T);

#[derive(Deref, DerefMut, Component)]
pub struct GeometryCtrl(DVec2);