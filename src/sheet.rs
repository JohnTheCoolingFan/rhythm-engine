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

struct Beat<'a, T> {
    start: P32,
    entity: &'a T,
    repeat: RepeaterAffinity,
}

#[derive(Default)]
struct Harmony<'a> {
    /// Exclusive
    spline: Option<Beat<'a, Spline>>,
    automation: Option<Beat<'a, Automation>>,
    /// Exclusive
    /// REQ: Some(_) = anchors
    color: Option<Beat<'a, Color>>,
    luminosity: Option<Beat<'a, Luminosity>>,
    scale: Option<Beat<'a, Scale>>,
    rotation: Option<Beat<'a, Rotation>>,
    /// Optional
    /// REQ: Some(_) = anchors && Some(_) = (rotation | scale)
    geometry_ctrl: Option<Beat<'a, GeometryCtrl>>,
}

#[rustfmt::skip]
#[derive(SystemParam)]
struct SheetParam<'w, 's, T: Component> {
    entities: Query<'w, 's, &'static T>,
    sheets: Query<'w, 's, (
        &'static SheetPosition,
        &'static Instance<T>,
        Option<&'static RepeaterAffinity>,
    )>,
}

#[rustfmt::skip]
impl<'w, 's, T: Component> SheetParam<'w, 's, T> {
    fn add_all<'a>(
        &'a self,
        time: SongTime,
        harmonies: &mut [Harmony<'a>],
        grabber: impl for<'b> Fn(&'b mut Harmony<'a>) -> &'b mut Option<Beat<'a, T>>
    ) {
        self.sheets
            .iter()
            .filter(|(pos, ..)| f32::EPSILON < pos.duration.raw())
            .filter(|(pos, ..)| pos.scheduled_at(*time))
            .for_each(|(pos, instance, affinity)| harmonies[pos.coverage()]
                .iter_mut()
                .for_each(|harmony| *grabber(harmony) = self
                    .entities
                    .get(**instance)
                    .ok()
                    .map(|entity| Beat {
                        entity,
                        start: pos.start,
                        repeat: affinity.copied().unwrap_or_default()
                    })
                )
            )
    }
}

#[derive(Clone, Copy)]
enum Modulation {
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
    splines: SheetParam<Spline>,
    automations: SheetParam<Automation>,
    colors: SheetParam<Color>,
    luminosities: SheetParam<Luminosity>,
    scales: SheetParam<Scale>,
    rotations: SheetParam<Rotation>,
    geometry_ctrls: SheetParam<GeometryCtrl>,
)
    -> [(Redirect, Modulation); MAX_CHANNELS]
{
    let harmonies = [(); MAX_CHANNELS].map(|_| Harmony::default()).tap_mut(|harmonies| {
        splines.add_all(
            *time,
            harmonies,
            |harmony| &mut harmony.spline
        );
        automations.add_all(
            *time,
            harmonies,
            |harmony| &mut harmony.automation
        );
        colors.add_all(
            *time,
            harmonies,
            |harmony| &mut harmony.color
        );
        luminosities.add_all(
            *time,
            harmonies,
            |harmony| &mut harmony.luminosity
        );
        scales.add_all(
            *time,
            harmonies,
            |harmony| &mut harmony.scale
        );
        rotations.add_all(
            *time,
            harmonies,
            |harmony| &mut harmony.rotation
        );
        geometry_ctrls.add_all(
            *time,
            harmonies,
            |harmony| &mut harmony.geometry_ctrl
        );
    });

    let mut modulations = sheet_inputs
        .into_iter()
        .zip(harmonies.into_iter())
        .map(|((response, repeater), harmony)| (
            response.redirect,
            match &harmony {
                Harmony { spline: Some(Beat { start, entity, repeat }), .. } => entity.play(
                    *start - if **repeat { repeater.repeat_time } else { response.seek_time }
                ),
                Harmony { automation , .. } if automation.is_some() => {
                    match harmony {
                        Harmony {
                            color: Some(Beat { start, entity, repeat }),
                            ..
                        } => {
                            todo!()
                        }
                        Harmony {
                            luminosity: Some(Beat { start, entity, repeat }),
                            ..
                        } => {
                            todo!()
                        }
                        Harmony {
                            scale: Some(Beat { start, entity, repeat }),
                            geometry_ctrl,
                            ..
                        } => {
                            todo!()
                        }
                        Harmony {
                            rotation: Some(Beat { start, entity, repeat }),
                            geometry_ctrl,
                            ..
                        } => {
                            todo!()
                        }
                        _ => Modulation::Nil
                    }
                }
                _ => Modulation::Nil
            }
        ));

    [(); MAX_CHANNELS].map(|_| modulations.next().unwrap())
}
