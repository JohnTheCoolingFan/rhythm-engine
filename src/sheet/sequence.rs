use std::marker::PhantomData;

use bevy::prelude::*;
use derive_more::{Deref, DerefMut};
use noisy_float::prelude::*;
use tap::Pipe;

use super::automation::*;
use crate::{
    sheet::{spline::*, *},
    utils::*,
};

struct GeometryCtrl(Vec2);

#[derive(Deref, DerefMut, Default, Clone, Copy)]
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

pub struct MarkerLuminosity;
pub struct MarkerRotation;
pub struct MarkerScale;

pub type Luminosity = Scalar<MarkerLuminosity, T32>;
pub type Rotation = Scalar<MarkerRotation>;
pub type Scale = Scalar<MarkerScale>;

#[derive(Deref, DerefMut, Default, Clone, Copy)]
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

#[derive(Component)]
struct TopBound;

#[derive(Component)]
struct BottomBound;

#[derive(Deref, DerefMut, Component)]
pub struct Sequence<T: Default>(Automation<T>);

impl<T: Default + Clone + Copy + Lerp<Output = T>> Sequence<T> {
    pub fn play(&self, offset: P32) -> <T as Lerp>::Output {
        self.interp(offset).unwrap_or_else(|anchor| anchor.val)
    }
}

impl Sequence<Option<GenID<Spline>>> {
    #[rustfmt::skip]
    pub fn play<'a>(
        &'a self,
        t: T32,
        offset: P32,
        get: impl Fn(GenID<Spline>) -> Option<&'a Spline> + Copy
    )
        -> Option<Vec2>
    {
        match self.at_or_after(offset) {
            [single] => single.val.and_then(get).map(|spline| spline.play(t)),
            [prev, curr, ..] => offset
                .completion_ratio(prev.quantify(), curr.quantify())
                .raw()
                .pipe(|weight| prev
                    .val
                    .and_then(get)
                    .zip(curr.val.and_then(get))
                    .map(|(prev, curr)| prev.play(t).lerp(curr.play(t), weight))
                ),
            _ => panic!("Unexpected existing no item control table"),
        }
    }
}
