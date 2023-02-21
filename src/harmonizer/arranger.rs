use super::*;
use crate::{timing::*, utils::*, *};
use itertools::Itertools;
use tap::{Pipe, Tap};

#[derive(Clone, Copy, PartialEq)]
pub struct CoverageRange(u8, u8);

impl CoverageRange {
    pub fn new(start: u8, end: u8) -> Self {
        assert!(start <= end);
        CoverageRange(start, end)
    }

    pub fn contains(&self, value: u8) -> bool {
        (self.0..=self.1).contains(&value)
    }

    pub fn contiguous_union(&self, other: Self) -> Option<Self> {
        [(*self, other), (other, *self)]
            .into_iter()
            .map(|(a, b)| a.0 as i16 - b.1 as i16)
            .any(|diff| diff.abs() <= 1)
            .then(|| CoverageRange(self.0.min(other.0), self.1.max(other.1)))
    }
}

#[derive(Clone, Copy)]
pub struct Condensed;

impl Property<Vec<CoverageRange>> for Condensed {
    fn ensure(target: &mut Vec<CoverageRange>) {
        *target = target
            .tap_mut(|vec| vec.dedup())
            .tap_mut(|vec| vec.sort_by_key(|coverage| coverage.0))
            .iter()
            .copied()
            .coalesce(|prev, curr| prev.contiguous_union(curr).ok_or((prev, curr)))
            .collect::<Vec<_>>();
    }
}

#[derive(Deref, DerefMut, Clone, Component)]
pub struct ChannelCoverage(pub Ensured<Vec<CoverageRange>, Condensed>);

impl ChannelCoverage {
    pub fn iter(&self) -> impl '_ + Clone + Iterator<Item = usize> {
        self.0
            .iter()
            .flat_map(|CoverageRange(start, end)| (*start as usize)..=(*end as usize))
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
    pub offset: P32,
    pub primary: T,
    pub secondary: Option<T>,
}

pub type SequenceArrangements<T> = Table<Option<Arrangement<GenID<Sequence<T>>>>>;

#[rustfmt::skip]
pub type SequenceSheets<'w, 's, T> = Query<'w, 's, (
    &'static TemporalOffsets,
    &'static ChannelCoverage,
    &'static PrimarySequence<Sources<Sequence<T>>>,
    Option<&'static SecondarySequence<Sources<Sequence<T>>>>,
)>;

#[rustfmt::skip]
pub fn arrange_sequences<T: Default + Component>(
    mut arrangements: ResMut<SequenceArrangements<T>>,
    time_tables: ResMut<TimeTables>,
    instances: SequenceSheets<T>,
) {
    arrangements.fill_with(|| None);
    instances.iter().for_each(|(offsets, coverage, primary, secondary)| {
        coverage.iter().for_each(|index| arrangements[index] = time_tables
            .clamped_times[index]
            .offset
            .pipe(iter_once)
            .chain(iter_once(time_tables.seek_times[index]))
            .find(|time| offsets.playable_at(*time))
            .map(|time| Arrangement {
                offset: time - offsets.start,
                primary: primary.pick(*time_tables.delegations[index]),
                secondary: secondary.map(|sources| sources.pick(*time_tables.delegations[index])),
            })
        )
    })
}
