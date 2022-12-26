use crate::{hit::*, utils::*};
use bevy::prelude::*;

enum Timing {
    BPM(u16),
    Manual,
}

#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub struct ClampedTime {
    pub offset: P64,
    pub upper_clamp: T64,
    pub lower_clamp: T64,
}

impl ClampedTime {
    #[rustfmt::skip]
    pub fn new(offset: P64) -> Self {
        Self { offset, upper_clamp: t64(1.), lower_clamp: t64(0.) }
    }
}

#[derive(Resource)]
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

#[derive(Clone, Copy, Component)]
pub struct TemporalOffsets {
    pub start: P64,
    pub duration: P64,
}

impl TemporalOffsets {
    pub fn scheduled_at(&self, time: P64) -> bool {
        (self.start.raw()..(self.start + self.duration).raw()).contains(&time.raw())
    }

    pub fn playable_at(&self, time: P64) -> bool {
        f64::EPSILON < self.duration.raw() && self.scheduled_at(time)
    }
}
