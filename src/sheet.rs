mod automation;
mod repeater;
mod sequence;
mod spline;

use crate::{hit::*, utils::*, *};
use automation::*;
use repeater::*;
use sequence::*;
use spline::*;

use core::iter::once as iter_once;

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
    pub clamped_times: Table<ClampedTime>,
    pub delegations: Table<Delegated>,
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
    main: GenID<T>,
    delegation: Option<GenID<T>>,
}

impl<T> Sources<T> {
    #[rustfmt::skip]
    fn pick(&self, delegated: bool) -> GenID<T> {
        match self {
            Self { delegation: Some(delegation), .. } if delegated => *delegation,
            _ => self.main,
        }
    }
}

pub enum Modulation {
    Position(Vec2),
    Rgba([T32; 4]),
    Luminosity(T32),
    Scale { magnitude: R32, ctrl: Option<Vec2> },
    Rotation { theta: R32, ctrl: Option<Vec2> },
    None,
}

impl From<Vec2> for Modulation {
    fn from(point: Vec2) -> Self {
        Self::Position(point)
    }
}

impl From<Rgba> for Modulation {
    fn from(color: Rgba) -> Self {
        Self::Rgba(*color)
    }
}

impl From<Luminosity> for Modulation {
    fn from(luminosity: Luminosity) -> Self {
        Self::Luminosity(*luminosity)
    }
}

impl From<Scale> for Modulation {
    fn from(scale: Scale) -> Self {
        Self::Scale {
            magnitude: *scale,
            ctrl: None,
        }
    }
}

impl From<Rotation> for Modulation {
    fn from(theta: Rotation) -> Self {
        Self::Rotation {
            theta: *theta,
            ctrl: None,
        }
    }
}

struct Arrangement<T> {
    offset: P32,
    primary: T,
    secondary: Option<T>,
}

#[rustfmt::skip]
fn arrange<T: Default + Component>(
    mut arrangements: ResMut<Table<Option<Arrangement<GenID<T>>>>>,
    time_tables: ResMut<TimeTables>,
    instances: Query<(
        &Sheet,
        &PrimarySequence<Sources<T>>,
        Option<&SecondarySequence<Sources<T>>>,
        Option<&RepeaterAffinity>,
    )>,
) {
    arrangements.fill_with(|| None);
    instances.iter().for_each(|(sheet, primary, secondary, affinity)| {
        sheet.coverage().for_each(|index| arrangements[index] = affinity
            .map(|_| time_tables.clamped_times[index].offset)
            .into_iter()
            .chain(iter_once(time_tables.seek_times[index]))
            .find(|time| sheet.scheduled_at(*time))
            .map(|time| Arrangement {
                offset: time - sheet.start,
                primary: primary.pick(*time_tables.delegations[index]),
                secondary: secondary.map(|sources| sources.pick(*time_tables.delegations[index])),
            })
        )
    })
}

#[derive(SystemParam)]
struct Ensemble<'w, 's, T: Component> {
    sources: Query<'w, 's, &'static T>,
    arrangements: Res<'w, Table<Option<Arrangement<GenID<T>>>>>,
}

impl<'w, 's, T: Component> Ensemble<'w, 's, T> {
    #[rustfmt::skip]
    fn get(&self, channel: usize) -> Option<Arrangement<&T>> {
        self.arrangements[channel].as_ref().map(|arrangement| Arrangement {
            offset: arrangement.offset,
            primary: self.sources.get(*arrangement.primary).unwrap(),
            secondary: arrangement.secondary.map(|secondary| self
                .sources
                .get(*secondary)
                .unwrap()
            ),
        })
    }
}

impl<'w, 's> Ensemble<'w, 's, Sequence<Spline>> {
    #[rustfmt::skip]
    fn play(&self, channel: usize,  t: T32) -> Option<Modulation> {
        self.get(channel).and_then(|arrangement| arrangement
            .secondary
            .is_none()
            .then(|| arrangement.primary.play(t, arrangement.offset).into())
        )
    }
}

#[rustfmt::skip]
impl<'w, 's, T> Ensemble<'w, 's, Sequence<T>>
where
    T: Default + Component + Clone + Copy + Lerp<Output = T>,
    Modulation: From<T>,
{
    fn play(&self, channel: usize, t: T32) -> Option<Modulation> {
        self.get(channel).and_then(|arrangement| arrangement
            .secondary
            .map(|secondary| secondary.play(arrangement.offset))
            .map(|secondary| arrangement
                .primary
                .play(arrangement.offset)
                .lerp(&secondary, t)
                .into()
            )
        )
    }

    fn play_primary(&self, channel: usize) -> Option<Modulation> {
        self.get(channel).and_then(|arrangement| arrangement
            .secondary
            .is_none()
            .then(|| arrangement.primary.play(arrangement.offset).into())
        )
    }
}

#[derive(SystemParam)]
struct Performers<'w, 's> {
    splines: Ensemble<'w, 's, Sequence<Spline>>,
    colors: Ensemble<'w, 's, Sequence<Rgba>>,
    luminosities: Ensemble<'w, 's, Sequence<Luminosity>>,
    scales: Ensemble<'w, 's, Sequence<Scale>>,
    rotations: Ensemble<'w, 's, Sequence<Rotation>>,
}

#[rustfmt::skip]
fn harmonize(
    mut modulations: ResMut<Table<Option<Modulation>>>,
    time_tables: ResMut<TimeTables>,
    performers: Performers,
    geom_ctrl_sources: Query<&GeometryCtrl>,
    geom_ctrls: Query<(&Sheet, &GenID<GeometryCtrl>)>,
    automation_sources: Query<&Automation<T32>>,
    automations: Query<(&Sheet, &Sources<Automation<T32>>, Option<&RepeaterAffinity>)>,
) {
    let TimeTables { song_time, seek_times, clamped_times, delegations } = *time_tables;

    modulations.fill_with(|| None);

    automations.iter().for_each(|(sheet, automation, affinity)| {
        sheet.coverage().for_each(|index| {
            if let Some(t) = affinity
                .map(|_| clamped_times[index])
                .into_iter()
                .chain(iter_once(ClampedTime::new(seek_times[index])))
                .find(|ClampedTime { offset, .. }| sheet.scheduled_at(*offset))
                .and_then(|clamped_time| {
                    automation_sources
                        .get(*automation.pick(*delegations[index]))
                        .ok()
                        .map(|automation| automation.play(clamped_time))
                })
            {
                let performances = [
                    performers.splines.play(index, t),
                    performers.colors.play(index, t),
                    performers.luminosities.play(index, t),
                    performers.scales.play(index, t),
                    performers.rotations.play(index, t),
                ];

                modulations[index] = performances
                    .into_iter()
                    .find(Option::is_some)
                    .unwrap_or(Some(Modulation::None))
            }
        })
    });

    modulations.iter_mut().enumerate().for_each(|(index, modulation)| if let None = modulation {
        let performances = [
            performers.colors.play_primary(index),
            performers.luminosities.play_primary(index),
            performers.scales.play_primary(index),
            performers.rotations.play_primary(index),
        ];

        *modulation = performances
            .into_iter()
            .find(Option::is_some)
            .flatten()
    });

    geom_ctrls.iter().filter(|(sheet, ..)| sheet.scheduled_at(song_time)).for_each(|(sheet, id)| {
        sheet.coverage().for_each(|index| {
            use Modulation::*;
            if let Some(Scale { ctrl, .. } | Rotation { ctrl, .. }) = &mut modulations[index] {
                *ctrl = geom_ctrl_sources.get(**id).ok().map(|ctrl| **ctrl)
            }
        })
    })
}
