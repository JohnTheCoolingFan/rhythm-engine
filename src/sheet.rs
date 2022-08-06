mod automation;
mod bound_sequence;
mod repeater;
mod spline;

use bound_sequence::*;
use repeater::*;
use spline::*;

use crate::{hit::*, utils::*, *};
use std::{marker::PhantomData, ops::RangeInclusive};

use bevy::{ecs::system::SystemParam, prelude::*};
use derive_more::Deref;
use noisy_float::prelude::*;
use tap::tap::Tap;

#[derive(Clone, Copy)]
pub struct Coverage(u8, u8);

#[derive(Component)]
pub struct Sheet {
    pub start: P32,
    pub duration: P32,
    coverage: Coverage,
}

impl Sheet {
    pub fn coverage<T: From<u8>>(&self) -> RangeInclusive<T> {
        self.coverage.0.into()..=self.coverage.1.into()
    }

    pub fn span<T: From<u8>>(&self) -> T {
        (self.coverage.1 - self.coverage.0).into()
    }

    pub fn scheduled_at(&self, time: P32) -> bool {
        (self.start.raw()..(self.start + self.duration).raw()).contains(&time.raw())
    }

    #[rustfmt::skip]
    pub fn scheduled_in<'a>(&'a self, times: &'a [SeekTime]) -> impl Iterator<Item = usize> + '_ {
        self.coverage::<usize>()
            .take(if self.duration.raw() < f32::EPSILON { 0 } else { self.span() })
            .filter(|n| self.scheduled_at(*times[*n]))
    }
}

#[derive(Clone, Copy, Component, Deref)]
struct Primary<T>(T);

#[derive(Clone, Copy, Component, Deref)]
struct Secondary<T>(T);

#[derive(Component, Deref)]
pub struct GenID<T> {
    #[deref]
    id: Entity,
    _phantom: PhantomData<T>,
}

impl<T> GenID<T> {
    pub fn new(id: Entity) -> Self {
        Self {
            id,
            _phantom: PhantomData,
        }
    }
}

impl<T> Clone for GenID<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            _phantom: PhantomData,
        }
    }
}

impl<T> Copy for GenID<T> {}

type Automation = automation::Automation<T32>;
type Color = BoundSequence<Rgba>;
type Luminosity = BoundSequence<bound_sequence::Luminosity>;
type Scale = BoundSequence<bound_sequence::Scale>;
type Rotation = BoundSequence<bound_sequence::Rotation>;

#[derive(Clone, Copy)]
pub enum Modulation {
    Nil,
    Position(Vec2),
    Color(Rgba),
    Luminosity(T32),
    Scale {
        magnitude: R32,
        ctrl: Option<Vec2>,
    },
    Rotation {
        theta: R32,
        ctrl: Option<Vec2>,
    },
}

pub trait Synth {
    type Output;
    fn play_from(&self, offset: P32, lower_clamp: T32, upper_clamp: T32) -> Self::Output;
}

struct Arrangement<T> {
    gen_id: GenID<T>,
    index: usize,
    offset: P32,
    lower_clamp: T32,
    upper_clamp: T32,
}

impl<'w, 's, T: Component + Synth> Arrangement<T> {
    fn eval(self, sources: &Query<'w, 's, &T>) -> <T as Synth>::Output {
        sources.get(*self.gen_id).unwrap().play_from(
            self.offset,
            self.lower_clamp,
            self.upper_clamp,
        )
    }
}

#[rustfmt::skip]
fn arrange<T: Component>(
    mut arrangements: ResMut<Table<Option<Arrangement<T>>>>,
    seek_times: Res<Table<SeekTime>>,
    delegations: Res<Table<Delegated>>,
    repetitions: Res<Table<Repetition>>,
    instances: Query<(
        &Sheet,
        &RepeaterAffinity,
        &Primary<GenID<T>>,
        Option<&Secondary<GenID<T>>>,
    )>,
) {
    let repeat_times = (**repetitions).map(|Repetition { time, .. }| SeekTime(time));

    instances.iter().for_each(|(sheet, repeated, primary, secondary)| {
        let arranger = |repeated: bool| {
            let seek_times = &seek_times;
            let delegations = &delegations;
            let repetitions = &repetitions;
            move |index: usize| {
                let Repetition { time, lower_clamp, upper_clamp } = repeated
                    .then(|| repetitions[index])
                    .unwrap_or_else(|| Repetition {
                        time: *seek_times[index],
                        lower_clamp: t32(0.),
                        upper_clamp: t32(1.)
                    });

                Arrangement {
                    index,
                    offset: time - sheet.start,
                    lower_clamp,
                    upper_clamp,
                    gen_id: delegations[index]
                        .then(|| secondary)
                        .flatten()
                        .map_or(**primary, |secondary| **secondary)
                }
            }
        };

        Some(sheet.scheduled_in(&**seek_times).map(arranger(false)))
            .into_iter()
            .chain(repeated.then(|| sheet.scheduled_in(&repeat_times).map(arranger(true))))
            .flatten()
            .for_each(|item @ Arrangement { index, .. }| arrangements[index] = Some(item))
    });
}

#[derive(SystemParam)]
struct Composition<'w, 's, T: Component> {
    sources: Query<'w, 's, &'static T>,
    arrangements: Res<'w, Table<Option<Arrangement<T>>>>,
}

fn harmonize(
    // Harmonizer Params
    // Exclusive
    splines: Composition<Spline>,
    automations: Composition<Automation>,
    // Exclusive
    // REQ: Some(_) = automation
    colors: Composition<Color>,
    luminosities: Composition<Luminosity>,
    scales: Composition<Scale>,
    rotations: Composition<Rotation>,
    // Optional
    // REQ: Some(_) = automation && Some(_) = (rotation | scale)
    geometry_ctrls: Composition<GeometryCtrl>,
    // System params
    mut modulations: Res<Table<Modulation>>,
) {
    todo!()
}
