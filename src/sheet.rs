mod automation;
mod bound_sequence;
mod repeater;
mod spline;

use bound_sequence::*;
use repeater::*;
use spline::*;

use crate::{hit::*, utils::*, SongTime, MAX_CHANNELS};
use std::{
    marker::PhantomData,
    ops::{Deref, RangeInclusive},
};

use bevy::{ecs::system::SystemParam, prelude::*};
use noisy_float::prelude::*;
use tap::tap::Tap;

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

#[rustfmt::skip]
#[derive(SystemParam)]
struct Ensemble<'w, 's, T: Component> {
    entities: Query<'w, 's, &'static T>,
    sheets: Query<'w, 's, (
        &'static SheetPosition,
        &'static Instance<T>,
        &'static RepeaterAffinity,
    )>,
}

#[rustfmt::skip]
impl<'w, 's, T: Component> Ensemble<'w, 's, T> {
    fn add_all<'a>(
        &'a self,
        time: SongTime,
        arrangements: &mut [Arrangement<'a>],
        grabber: impl for<'b> Fn(&'b mut Arrangement<'a>) -> &'b mut Option<&'a T>,
    ) {
        self.sheets
            .iter()
            .filter(|(pos, ..)| f32::EPSILON < pos.duration.raw())
            .filter(|(pos, ..)| pos.scheduled_at(*time))
            .for_each(|(pos, instance, _)| arrangements[pos.coverage()]
                .iter_mut()
                .for_each(|arrangement| *grabber(arrangement) = self.entities.get(**instance).ok())
            )
    }
}

#[derive(Default)]
struct Arrangement<'a> {
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

#[rustfmt::skip]
fn produce_modulations(
    time: Res<SongTime>,
    In(sheet_inputs): In<[(ResponseOutput, RepeaterOutput); MAX_CHANNELS]>,
    splines: Ensemble<Spline>,
    automations: Ensemble<Automation>,
    colors: Ensemble<Color>,
    luminosities: Ensemble<Luminosity>,
    scales: Ensemble<Scale>,
    rotations: Ensemble<Rotation>,
    geometry_ctrls: Ensemble<GeometryCtrl>,
)
    -> [Modulation; MAX_CHANNELS]
{
    let arrangements = [(); MAX_CHANNELS].map(|_| Arrangement::default()).tap_mut(|arrangements| {
        splines.add_all(*time, arrangements, |arrangement| &mut arrangement.spline);
        automations.add_all(*time, arrangements, |arrangement| &mut arrangement.automation);
        colors.add_all(*time, arrangements, |arrangement| &mut arrangement.color);
        luminosities.add_all(*time, arrangements, |arrangement| &mut arrangement.luminosity);
        scales.add_all(*time, arrangements, |arrangement| &mut arrangement.scale);
        rotations.add_all(*time, arrangements, |arrangement| &mut arrangement.rotation);
        geometry_ctrls.add_all(*time, arrangements, |arrangement| &mut arrangement.geometry_ctrl);
    });

    todo!()
}
