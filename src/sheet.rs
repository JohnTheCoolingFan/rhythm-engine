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
use tap::Pipe;

#[derive(Clone, Copy)]
pub struct Coverage(u8, u8);

#[derive(Component)]
pub struct Sheet {
    pub start: P32,
    pub duration: P32,
    coverage: Coverage,
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

impl<T> Copy for GenID<T> {}

impl<T> Clone for GenID<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            _phantom: PhantomData,
        }
    }
}

#[derive(Clone, Copy)]
pub enum Modulation {
    Nil,
    Position(Vec2),
    Color([T32; 4]),
    Luminosity(T32),
    Scale { magnitude: R32, ctrl: Option<Vec2> },
    Rotation { theta: R32, ctrl: Option<Vec2> },
}

pub trait Synth {
    type Output;
    fn play(&self, offset: P32, lower_clamp: T32, upper_clamp: T32) -> Self::Output;
}

struct Arrangement<T> {
    entity: T,
    offset: P32,
    lower_clamp: T32,
    upper_clamp: T32,
}

impl<T, S> Arrangement<T>
where
    T: std::ops::Deref<Target = S>,
    S: Synth,
{
    #[rustfmt::skip]
    fn play(&self) -> S::Output {
        self.entity.play(self.offset, self.lower_clamp, self.upper_clamp)
    }
}

#[rustfmt::skip]
fn arrange<T: Component>(
    mut arrangements: ResMut<Table<Option<Arrangement<GenID<T>>>>>,
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
    arrangements.fill_with(|| None);

    instances.iter().for_each(|(sheet, affinity, primary, secondary)| {
        let regular = sheet
            .active_in(&**seek_times, |seek_time| **seek_time)
            .map(|index| (index, Arrangement {
                lower_clamp: t32(0.),
                upper_clamp: t32(1.),
                offset: *seek_times[index] - sheet.start,
                entity: delegations[index]
                    .then(|| secondary)
                    .flatten()
                    .map_or(**primary, |secondary| **secondary)
            }));

        let repeated = sheet
            .active_in(&**repetitions, |repetition| repetition.time)
            .map(|index| (index, repetitions[index]))
            .map(|(index, Repetition { time, lower_clamp, upper_clamp })| (index, Arrangement {
                lower_clamp,
                upper_clamp,
                offset: time - sheet.start,
                entity: delegations[index]
                    .then(|| secondary)
                    .flatten()
                    .map_or(**primary, |secondary| **secondary)
            }));

        regular.chain(repeated.take_while(|_| **affinity)).for_each(|(index, arrangement)| {
            arrangements[index] = Some(arrangement)
        })
    });
}

#[derive(SystemParam)]
struct Composition<'w, 's, T: Component> {
    sources: Query<'w, 's, &'static T>,
    arrangements: Res<'w, Table<Option<Arrangement<GenID<T>>>>>,
}

#[rustfmt::skip]
impl<'w, 's, T: Component> Composition<'w, 's, T> {
    fn get(&self, index: usize) -> Option<Arrangement<&T>> {
        self.arrangements[index]
            .as_ref()
            .map(|&Arrangement { entity, lower_clamp, upper_clamp, offset }| Arrangement {
                offset,
                lower_clamp,
                upper_clamp,
                entity: self.sources.get(*entity).unwrap(),
            })
    }
}

type Automation = automation::Automation<T32>;
type Color = BoundSequence<Rgba>;
type Luminosity = BoundSequence<bound_sequence::Luminosity>;
type Scale = BoundSequence<bound_sequence::Scale>;
type Rotation = BoundSequence<bound_sequence::Rotation>;

struct Ensemble<'a> {
    spline: Option<Arrangement<&'a Spline>>,
    automation: Option<Arrangement<&'a Automation>>,
    color: Option<Arrangement<&'a Color>>,
    luminosity: Option<Arrangement<&'a Luminosity>>,
    scale: Option<Arrangement<&'a Scale>>,
    rotation: Option<Arrangement<&'a Rotation>>,
    geom_ctrl: Option<Arrangement<&'a GeometryCtrl>>,
}

#[rustfmt::skip]
fn harmonize(
    splines: Composition<Spline>,
    automations: Composition<Automation>,
    colors: Composition<Color>,
    luminosities: Composition<Luminosity>,
    scales: Composition<Scale>,
    rotations: Composition<Rotation>,
    geometry_ctrls: Composition<GeometryCtrl>,
    mut modulations: ResMut<Table<Modulation>>,
) {
    modulations.iter_mut().enumerate().for_each(|(index, modulation)| {
        let ensemble = Ensemble {
            spline: splines.get(index),
            automation: automations.get(index),
            color: colors.get(index),
            luminosity: luminosities.get(index),
            scale: scales.get(index),
            rotation: rotations.get(index),
            geom_ctrl: geometry_ctrls.get(index)
        };

        *modulation = match ensemble {
            // Spline
            Ensemble { spline: Some(spline), .. } => spline.play(),

            // Color
            Ensemble { automation: Some(clip), color: Some(color), .. } => color
                .play()
                .pipe(|(lower, upper)| lower.lerp(&upper, clip.play()))
                .pipe(|color| Modulation::Color(*color)),

            // Luminosity
            Ensemble { automation: Some(clip), luminosity: Some(lumin), .. } => lumin
                .play()
                .pipe(|(lower, upper)| lower.lerp(&upper, clip.play()))
                .pipe(|luminosity| Modulation::Luminosity(*luminosity)),

            // Rotation
            Ensemble { automation: Some(clip), rotation: Some(rotation), geom_ctrl, .. } => rotation
                .play()
                .pipe(|(lower, upper)| lower.lerp(&upper, clip.play()))
                .pipe(|rotation| Modulation::Rotation {
                    theta: *rotation,
                    ctrl: geom_ctrl.map(|arrangement| **arrangement.entity),
                }),

            // Scale
            Ensemble { automation: Some(clip), scale: Some(scale), geom_ctrl, .. } => scale
                .play()
                .pipe(|(lower, upper)| lower.lerp(&upper, clip.play()))
                .pipe(|scale| Modulation::Scale {
                    magnitude: *scale,
                    ctrl: geom_ctrl.map(|arrangement| **arrangement.entity),
                }),

            // No Harmony
            _ => Modulation::Nil
        }
    })
}
