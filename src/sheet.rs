mod automation;
mod bound_sequence;
mod repeater;
mod spline;

use bound_sequence::*;
use repeater::*;
use spline::*;

use crate::{hit::*, utils::*, SongTime};
use std::{marker::PhantomData, ops::RangeInclusive};

use bevy::{ecs::system::SystemParam, prelude::*};
use noisy_float::prelude::*;

type Automation = automation::Automation<T32>;
type Color = BoundSequence<SpannedBound<Rgba>>;
type Luminosity = BoundSequence<SpannedBound<bound_sequence::Luminosity>>;
type Scale = BoundSequence<ScalarBound<bound_sequence::Scale>>;
type Rotation = BoundSequence<ScalarBound<bound_sequence::Rotation>>;

#[derive(Clone, Copy)]
pub struct Coverage(u8, u8);

#[derive(Component)]
pub struct SheetPosition {
    pub start: P32,
    pub duration: P32,
    coverage: Coverage,
}

impl SheetPosition {
    pub fn coverage<T: From<u8>>(&self) -> RangeInclusive<T> {
        self.coverage.0.into()..=self.coverage.1.into()
    }
}

impl SheetPosition {
    pub fn scheduled_at(&self, time: P32) -> bool {
        (self.start.raw()..self.start.raw() + self.duration.raw()).contains(&time.raw())
    }
}

#[derive(Clone, Copy, Component)]
pub struct Instance<T> {
    pub entity: Entity,
    _phantom: PhantomData<T>,
}

/*#[derive(Default)]
struct Ensemble<'a> {
    /// Alawys valid
    hit_response: Option<Instance<&'a HitResponse>>,
    repeater: Option<Instance<&'a Repeater>>,
    /// Exclusive
    spline: Option<Instance<&'a Spline>>,
    automation: Option<Instance<&'a Automation<T32>>>,
    /// Exclusive
    /// REQ: Some(_) = anchors
    color: Option<Instance<&'a BoundSequence<SpannedBound<Rgba>>>>,
    luminosity: Option<Instance<&'a BoundSequence<SpannedBound<Luminosity>>>>,
    scale: Option<Instance<&'a BoundSequence<ScalarBound<Scale>>>>,
    rotation: Option<Instance<&'a BoundSequence<ScalarBound<Rotation>>>>,
    /// Optional
    /// REQ: Some(_) = anchors && Some(_) = (rotation | scale)
    geometry_ctrl: Option<Instance<&'a GeometryCtrl>>,
}*/

enum Modulation {
    Position(Vec2),
    Color(Rgba),
    Luminosity(Luminosity),
    Scale {
        magnitude: R32,
        ctrl: Option<Vec2>,
    },
    Rotation {
        theta: R32,
        ctrl: Option<Vec2>,
    },
}
