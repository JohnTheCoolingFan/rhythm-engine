use super::{repeater::*, *};
use crate::{timing::*, utils::*, *};
use std::ops::RangeInclusive;

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

pub type SequenceArrangements<T> = Table<Option<Arrangement<GenID<Sequence<T>>>>>;

#[rustfmt::skip]
pub type SequenceSheets<'w, 's, T> = Query<'w, 's, (
    &'static Sheet,
    &'static PrimarySequence<Sources<Sequence<T>>>,
    Option<&'static SecondarySequence<Sources<Sequence<T>>>>,
    Option<&'static RepeaterAffinity>,
)>;

#[rustfmt::skip]
pub fn arrange_sequences<T: Default + Component>(
    mut arrangements: ResMut<SequenceArrangements<T>>,
    time_tables: ResMut<TimeTables>,
    instances: SequenceSheets<T>,
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
