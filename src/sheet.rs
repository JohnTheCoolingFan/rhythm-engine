mod automation;
mod repeater;
mod sequence;
mod spline;

use repeater::*;
use sequence::*;
use spline::*;

use crate::{hit::*, utils::*, *};

use std::{marker::PhantomData, ops::RangeInclusive};

use bevy::{ecs::system::SystemParam, prelude::*};
use derive_more::Deref;
use noisy_float::prelude::*;
use tap::Pipe;

pub const MAX_CHANNELS: usize = 256;

#[derive(Deref, DerefMut, From, Clone, Copy)]
pub struct Table<T>([T; MAX_CHANNELS]);

impl<T> Table<T> {
    pub fn fill_with(&mut self, func: impl Fn() -> T) {
        self.0 = [(); MAX_CHANNELS].map(|_| func());
    }
}

pub struct TimeTables {
    pub song_time: P32,
    pub seek_times: Table<P32>,
    pub delegations: Table<Delegated>,
    pub repetitions: Table<Repetition>,
}

#[derive(Clone, Copy)]
pub struct Coverage(pub u8, pub u8);

#[derive(Component)]
pub struct Sheet {
    pub start: P32,
    pub duration: P32,
    pub coverage: Coverage,
}

impl Sheet {
    pub fn coverage(&self) -> RangeInclusive<usize> {
        self.coverage.0.into()..=self.coverage.1.into()
    }

    pub fn span<T: From<u8>>(&self) -> T {
        (self.coverage.1 - self.coverage.0).into()
    }

    pub fn scheduled_at(&self, time: P32) -> bool {
        (self.start.raw()..(self.start + self.duration).raw()).contains(&time.raw())
    }

    #[rustfmt::skip]
    pub fn active_in<'a, T>(
        &'a self,
        items: &'a [T],
        key: fn(&'a T) -> P32
    )
        -> impl Iterator<Item = usize> + '_
    {
        self.coverage()
            .take(if self.duration.raw() < f32::EPSILON { 0 } else { self.span() })
            .filter(move |index| self.scheduled_at(key(&items[*index])))
    }
}

#[derive(Component, Deref)]
pub struct GenID<T> {
    #[deref]
    id: Entity,
    _phantom: PhantomData<T>,
}

#[rustfmt::skip]
impl<T> GenID<T> {
    pub fn new(id: Entity) -> Self {
        Self { id, _phantom: PhantomData }
    }
}

#[rustfmt::skip]
impl<T> Clone for GenID<T> {
    fn clone(&self) -> Self {
        Self { id: self.id, _phantom: PhantomData }
    }
}

impl<T> Copy for GenID<T> {}

#[derive(Clone, Copy, Component)]
struct Sources<T> {
    primary: GenID<T>,
    secondary: Option<GenID<T>>,
}

impl<T: Component> Sources<T> {
    #[rustfmt::skip]
    fn get(self, delegate: bool) -> Entity {
        match self {
            Self { secondary: Some(gen_id), .. } if delegate => *gen_id,
            Self { primary: gen_id, .. } => *gen_id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Modulation {
    Position(Vec2),
    Color([T32; 4]),
    Luminosity(T32),
    Scale { magnitude: R32, ctrl: Option<Vec2> },
    Rotation { theta: R32, ctrl: Option<Vec2> },
}
