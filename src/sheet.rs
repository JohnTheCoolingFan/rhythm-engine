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

pub struct TimeTable {
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

pub trait Synth {
    type Output;
    fn play(&self, offset: P32, lower_clamp: T32, upper_clamp: T32) -> Self::Output;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Modulation {
    Position(Vec2),
    Color([T32; 4]),
    Luminosity(T32),
    Scale { magnitude: R32, ctrl: Option<Vec2> },
    Rotation { theta: R32, ctrl: Option<Vec2> },
    Partial {},
}

pub enum SynthProduce {}
/*struct Arrangement<T> {
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

type Automation = automation::Automation<T32>;
type Color = BoundSequence<Rgba>;
type Luminosity = BoundSequence<bound_sequence::Luminosity>;
type Scale = BoundSequence<bound_sequence::Scale>;
type Rotation = BoundSequence<bound_sequence::Rotation>;

#[rustfmt::skip]
fn harmonize(
    mut modulations: ResMut<Table<Modulation>>,
    automations: Query<(
        &Sheet,
        Option<&RepeaterAffinity>,
        &Sources<Automation>,
    )>,
    synths: Query<(
        &Sheet,
        Option<&RepeaterAffinity>,
        AnyOf<(
            &Sources<Spline>,
            &Sources<Color>,
            &Sources<Luminosity>,
            &Sources<Scale>,
            &Sources<Rotation>,
            &Sources<GeometryCtrl>,
        )>,
    )>,
) {
    modulations.iter_mut().enumerate().for_each(|(index, modulation)| {
        *modulation = match (Ensemble {
            spline: splines.get(index),
            automation: automations.get(index),
            color: colors.get(index),
            luminosity: luminosities.get(index),
            scale: scales.get(index),
            rotation: rotations.get(index),
            geom_ctrl: geometry_ctrls.get(index)
        }) {
            // Spline
            Ensemble { automation: Some(clip), spline: Some(spline), .. } => spline
                .entity
                .play(clip.play()),

            // Color
            Ensemble { automation: Some(clip), color: Some(color), .. } => color
                .play()
                .pipe(|(lower, upper)| *lower.lerp(&upper, clip.play()))
                .pipe(Modulation::Color),

            // Luminosity
            Ensemble { automation: Some(clip), luminosity: Some(luminosity), .. } => luminosity
                .play()
                .pipe(|(lower, upper)| *lower.lerp(&upper, clip.play()))
                .pipe(Modulation::Luminosity),

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
}*/
