use super::{repeater::*, *};
use crate::{hit::*, utils::*, *};
use std::ops::RangeInclusive;

use derive_more::Deref;

pub const MAX_CHANNELS: usize = 256;

#[derive(Deref, DerefMut, From, Clone, Copy)]
pub struct Table<T>(pub [T; MAX_CHANNELS]);

impl<T> Table<T> {
    pub fn fill_with(&mut self, func: impl Fn() -> T) {
        self.0 = [(); MAX_CHANNELS].map(|_| func());
    }
}

impl<T: Default> Default for Table<T> {
    fn default() -> Self {
        Self([(); MAX_CHANNELS].map(|_| T::default()))
    }
}

pub struct TimeTables {
    pub song_time: P64,
    pub seek_times: Table<P64>,
    pub clamped_times: Table<ClampedTime>,
    pub delegations: Table<Delegated>,
}

impl Default for TimeTables {
    fn default() -> Self {
        TimeTables {
            song_time: p64(0.),
            seek_times: Table([(); MAX_CHANNELS].map(|_| p64(0.))),
            delegations: Table([(); MAX_CHANNELS].map(|_| Delegated(false))),
            clamped_times: Table([(); MAX_CHANNELS].map(|_| ClampedTime::new(p64(0.)))),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Coverage(pub u8, pub u8);

#[derive(Component)]
pub struct Sheet {
    pub start: P64,
    pub duration: P64,
    pub coverage: Coverage,
}

impl Sheet {
    pub fn coverage(&self) -> RangeInclusive<usize> {
        self.coverage.0.into()..=self.coverage.1.into()
    }

    pub fn span<T: From<u8>>(&self) -> T {
        (self.coverage.1 - self.coverage.0).into()
    }

    pub fn scheduled_at(&self, time: P64) -> bool {
        (self.start.raw()..(self.start + self.duration).raw()).contains(&time.raw())
    }

    pub fn playable_at(&self, time: P64) -> bool {
        f64::EPSILON < self.duration.raw() && self.scheduled_at(time)
    }
}

#[derive(Clone, Copy, Component)]
pub struct Sources<T> {
    main: GenID<T>,
    delegation: Option<GenID<T>>,
}

impl<T> Sources<T> {
    #[rustfmt::skip]
    pub fn pick(&self, delegated: bool) -> GenID<T> {
        match self {
            Self { delegation: Some(delegation), .. } if delegated => *delegation,
            _ => self.main,
        }
    }
}

pub struct Arrangement<T> {
    pub offset: P64,
    pub primary: T,
    pub secondary: Option<T>,
}

#[rustfmt::skip]
pub fn arrange<T: Default + Component>(
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
            .find(|time| sheet.playable_at(*time))
            .map(|time| Arrangement {
                offset: time - sheet.start,
                primary: primary.pick(*time_tables.delegations[index]),
                secondary: secondary.map(|sources| sources.pick(*time_tables.delegations[index])),
            })
        )
    })
}
