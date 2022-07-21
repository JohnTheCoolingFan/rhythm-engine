mod automation;
mod bound_sequence;
mod repeater;
mod spline;

use bound_sequence::*;
use repeater::*;
use spline::*;

use crate::{hit::*, utils::*, SongTime};
use std::{
    marker::PhantomData,
    ops::{Deref, RangeInclusive},
};

use bevy::{ecs::system::SystemParam, prelude::*};
use noisy_float::prelude::*;

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

    pub fn scheduled_at(&self, time: P32) -> bool {
        (self.start.raw()..self.start.raw() + self.duration.raw()).contains(&time.raw())
    }
}

#[derive(Clone, Copy, Component)]
pub struct Instance<T> {
    entity: Entity,
    _phantom: PhantomData<T>,
}

impl<T> Instance<T> {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            _phantom: PhantomData,
        }
    }
}

impl<T> Deref for Instance<T> {
    type Target = Entity;
    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}

type Automation = automation::Automation<T32>;
type Color = BoundSequence<Rgba>;
type Luminosity = BoundSequence<bound_sequence::Luminosity>;
type Scale = BoundSequence<bound_sequence::Scale>;
type Rotation = BoundSequence<bound_sequence::Rotation>;

#[derive(Default)]
struct Ensemble<'a> {
    /// Exclusive
    spline: Option<&'a Spline>,
    automation: Option<&'a Automation>,
    /// Exclusive
    /// REQ: Some(_) = anchors
    color: Option<&'a Color>,
    luminosity: Option<&'a Luminosity>,
    scale: Option<&'a Scale>,
    rotation: Option<&'a Rotation>,
    /// Optional
    /// REQ: Some(_) = anchors && Some(_) = (rotation | scale)
    geometry_ctrl: Option<&'a GeometryCtrl>,
}

#[derive(Default, Clone, Copy)]
enum Modulation {
    #[default]
    Nil,
    Position(Vec2),
    Color(Rgba),
    Luminosity(T32),
    Scale {
        magnitude: R32,
        ctrl: Option<Vec2>,
    },
    Rotation {
        theta: R32,
        ctrl: Option<Vec2>,
    },
}

fn produce_modulations(
    time: Res<SongTime>,
    In(sheet_inputs): In<[(ResponseOutput, RepeaterOutput); 256]>,
    splines: Query<&Spline>,
    automations: Query<&Automation>,
    colors: Query<&Color>,
    luminosities: Query<&Luminosity>,
    scales: Query<&Scale>,
    rotations: Query<&Rotation>,
    geometry_ctrls: Query<&GeometryCtrl>,
) {
}
