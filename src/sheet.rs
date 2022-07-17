mod automation;
mod bound_sequence;
mod repeater;
mod spline;

use bound_sequence::*;
use repeater::*;
use spline::*;

use crate::{hit::*, utils::*, SongTime};
use std::marker::PhantomData;

use bevy::{ecs::system::SystemParam, prelude::*};
use noisy_float::prelude::*;
use tap::tap::Tap;

type Automation = automation::Automation<T32>;
type Color = BoundSequence<SpannedBound<Rgba>>;
type Luminosity = BoundSequence<SpannedBound<bound_sequence::Luminosity>>;
type Scale = BoundSequence<ScalarBound<bound_sequence::Scale>>;
type Rotation = BoundSequence<ScalarBound<bound_sequence::Rotation>>;

#[derive(Clone, Copy)]
struct Coverage {
    first: u8,
    last: u8,
}

#[derive(Component)]
struct SheetPosition {
    start: P32,
    duration: P32,
    coverage: Coverage,
}

impl SheetPosition {
    fn playable(&self, time: P32) -> bool {
        (self.start.raw()..self.start.raw() + self.duration.raw()).contains(&time.raw())
    }
}

#[derive(Clone, Copy, Component)]
pub struct Instance<T> {
    entity: Entity,
    _phantom: PhantomData<T>,
}

#[rustfmt::skip]
fn clear_hit_responses<T: Component>(
    song_time: Res<SongTime>,
    mut response_sheets: Query<(&SheetPosition, &mut ResponseState)>,
) {
    response_sheets
        .iter_mut()
        .filter(|(sheet, _)| !sheet.playable(**song_time))
        .for_each(|(_, mut response_state)| *response_state = ResponseState::Nil);
}

#[rustfmt::skip]
fn respond_to_hits<T: Component>(
    song_time: Res<SongTime>,
    hit_register: Res<HitRegister>,
    hit_responses: Query<&HitResponse>,
    mut response_sheets: Query<(
        &SheetPosition,
        &Instance<HitResponse>,
        &mut ResponseState
    )>,
)
    -> [(P32, Option<u8>); 256]
{
    [(**song_time, None); 256].tap_mut(|outputs| {
        response_sheets
            .iter_mut()
            .filter(|(sheet, _, _)| sheet.playable(**song_time))
            .for_each(|(sheet, instance, mut response_state)| {
                use ResponseKind::*;
                use ResponseState::*;

                let (Coverage { first, last }, HitResponse { kind, layer }) = (
                    sheet.coverage,
                    hit_responses.get(instance.entity).unwrap()
                );

                (first..=last).for_each(|index| {
                    let (seek_time, redirect) = &mut outputs[index as usize];

                    hit_register.iter().flatten().filter(|hit| hit.layer == *layer).for_each(|hit|
                        match (kind, &mut *response_state) {
                            (Commence | Switch(_), state) => *state = Delegated(true),
                            (Toggle(_), Delegated(delegate)) => *delegate = !*delegate,
                            (Toggle(_), state) => *state = Delegated(true),
                            (Follow(_), last_hit) => *last_hit = Hit(hit.object_time),
                            _ => {}
                        }
                    );

                    *redirect = match (kind, &mut *response_state) {
                        (Switch(shift) | Toggle(shift), Delegated(true)) => {
                            Some(index.wrapping_add(*shift))
                        }
                        _ => None
                    };

                    *seek_time = match (kind, &mut *response_state) {
                        (Commence, Delegated(delegate)) if !*delegate => {
                            sheet.start
                        }
                        (Follow(ex), Hit(hit)) if !(*hit..*hit + ex).contains(&**song_time) => {
                            *hit + ex
                        }
                        _ => **song_time
                    };
                })
            }
        )
    })
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
