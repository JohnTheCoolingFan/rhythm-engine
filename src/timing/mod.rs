use crate::{hit::*, utils::*};
use bevy::prelude::*;

enum Timing {
    BPM(u16),
    Manual,
}

#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub struct ClampedTime {
    pub offset: P32,
    pub upper_clamp: T32,
    pub lower_clamp: T32,
}

impl ClampedTime {
    pub fn new(offset: P32) -> Self {
        Self {
            offset,
            upper_clamp: t32(1.),
            lower_clamp: t32(0.),
        }
    }
}

#[derive(Resource)]
pub struct TimeTables {
    pub song_time: P32,
    pub seek_times: Table<P32>,
    pub clamped_times: Table<ClampedTime>,
    pub delegations: Table<Delegated>,
}

impl Default for TimeTables {
    fn default() -> Self {
        TimeTables {
            song_time: p32(0.),
            seek_times: Table([(); MAX_CHANNELS].map(|_| p32(0.))),
            delegations: Table([(); MAX_CHANNELS].map(|_| Delegated(false))),
            clamped_times: Table([(); MAX_CHANNELS].map(|_| ClampedTime::new(p32(0.)))),
        }
    }
}

#[derive(Clone, Component)]
pub struct TemporalOffsets {
    pub start: P32,
    pub duration: P32,
}

impl TemporalOffsets {
    pub fn scheduled_at(&self, time: P32) -> bool {
        (self.start.raw()..(self.start + self.duration).raw()).contains(&time.raw())
    }

    pub fn playable_at(&self, time: P32) -> bool {
        f32::EPSILON < self.duration.raw() && self.scheduled_at(time)
    }
}
