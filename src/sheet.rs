mod automation;
mod bound_sequence;
mod repeater;
mod spline;

use bound_sequence::*;
use repeater::*;
use spline::*;

use crate::{hit::*, utils::*, *};
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
pub struct Sheet {
    pub start: P32,
    pub duration: P32,
    coverage: Coverage,
}

impl Sheet {
    pub fn coverage<T: From<u8>>(&self) -> RangeInclusive<T> {
        self.coverage.0.into()..=self.coverage.1.into()
    }

    pub fn scheduled_at(&self, time: P32) -> bool {
        (self.start.raw()..self.start.raw() + self.duration.raw()).contains(&time.raw())
    }
}

#[derive(Clone, Copy, Component)]
pub struct GenID<T> {
    id: Entity,
    _phantom: PhantomData<T>,
}

impl<T> GenID<T> {
    pub fn new(id: Entity) -> Self {
        Self {
            id,
            _phantom: PhantomData,
        }
    }
}

impl<T> Deref for GenID<T> {
    type Target = Entity;
    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

type Automation = automation::Automation<T32>;
type Color = BoundSequence<Rgba>;
type Luminosity = BoundSequence<bound_sequence::Luminosity>;
type Scale = BoundSequence<bound_sequence::Scale>;
type Rotation = BoundSequence<bound_sequence::Rotation>;

struct Beat<T> {
    start: P32,
    gen_id: GenID<T>,
    repeat: RepeaterAffinity,
}

#[derive(SystemParam)]
struct Harmony<'w, 's> {
    /// Exclusive
    spline: Res<'w, Table<Option<Beat<Spline>>>>,
    automation: Res<'w, Table<Option<Beat<Automation>>>>,
    /// Exclusive
    /// REQ: Some(_) = anchors
    color: Res<'w, Table<Option<Beat<Color>>>>,
    luminosity: Res<'w, Table<Option<Beat<Luminosity>>>>,
    scale: Res<'w, Table<Option<Beat<Scale>>>>,
    rotation: Res<'w, Table<Option<Beat<Rotation>>>>,
    /// Optional
    /// REQ: Some(_) = anchors && Some(_) = (rotation | scale)
    geometry_ctrl: Res<'w, Table<Option<Beat<GeometryCtrl>>>>,
    #[system_param(ignore)]
    _phantom: PhantomData<&'s ()>,
}

/*#[rustfmt::skip]
#[derive(SystemParam)]
struct SheetParam<'w, 's, T: Component> {
    entities: Query<'w, 's, &'static T>,
    sheets: Query<'w, 's, (
        &'static Sheet,
        &'static GenID<T>,
        Option<&'static RepeaterAffinity>,
    )>,
}

#[rustfmt::skip]
impl<'w, 's, T: Component> SheetParam<'w, 's, T> {
    fn add_all<'a>(
        &'a self,
        seek_times: [P32; MAX_CHANNELS],
        harmonies: &mut [Harmony<'a>],
        grabber: impl for<'b> Fn(&'b mut Harmony<'a>) -> &'b mut Option<Beat<'a, T>>
    ) {
        self.sheets
            .iter()
            .filter(|(sheet, ..)| f32::EPSILON < sheet.duration.raw())
            .for_each(|(sheet, gen_id, affinity)| harmonies[sheet.coverage()]
                .iter_mut()
                .zip(seek_times[sheet.coverage()].iter())
                .filter(|(seek_time, ..)| sheet.scheduled_at(*time))
                .for_each(|harmony| *grabber(harmony) = self
                    .entities
                    .get(**gen_id)
                    .ok()
                    .map(|entity| Beat {
                        entity,
                        start: sheet.start,
                        repeat: affinity.copied().unwrap_or_default()
                    })
                )
            )
    }
}*/

#[derive(Clone, Copy)]
pub enum Modulation {
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

pub trait Synth {
    type Output;
    fn play_from(&self, offset: P32, repetition: Option<Repetition>) -> Self::Output;
}

/*#[rustfmt::skip]
fn produce_modulations(
    time: Res<SongTime>,
    In(sheet_inputs): In<[(Signal, Repetition); MAX_CHANNELS]>,
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
        .iter()
        .zip(harmonies.iter())
        .map(|((signal, repetition), harmony)| (
            signal.redirect,
            match &harmony {
                Harmony { spline: Some(Beat { start, entity, repeat }), .. } => {
                    todo!()
                }
                Harmony { automation: Some(Beat { start, entity, repeat }) , .. } => {
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
}*/
