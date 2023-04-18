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

#[derive(Clone, Copy, Default, Deref, DerefMut, PartialEq, Eq, Debug, Resource)]
pub struct SongTime(pub P32);

#[derive(Clone, Copy, Default, Deref, DerefMut, PartialEq, Eq, Debug)]
pub struct SeekTime(pub P32);

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
