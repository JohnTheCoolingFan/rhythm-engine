use super::automation::Weight;
use crate::{hit::*, sheet::*, utils::*};

use bevy::prelude::*;
use noisy_float::prelude::*;
use tap::tap::Tap;

pub struct RepeaterClamp {
    start: T32,
    end: T32,
    weight: Weight,
}

impl RepeaterClamp {
    pub fn eval(&self, t: T32) -> T32 {
        self.start.lerp(&self.end, self.weight.eval(t))
    }
}

#[derive(Component)]
pub struct Repeater {
    ping_pong: bool,
    period: P32,
    ceil: RepeaterClamp,
    floor: RepeaterClamp,
}

#[derive(Default, Deref, Component, Clone, Copy)]
pub struct RepeaterAffinity(bool);

#[derive(Clone, Copy)]
pub struct Repetition {
    pub time: P32,
    pub lower_clamp: T32,
    pub upper_clamp: T32,
}

impl Repetition {
    fn new(seek_time: P32) -> Self {
        Self {
            time: seek_time,
            lower_clamp: t32(0.),
            upper_clamp: t32(1.),
        }
    }
}

#[rustfmt::skip]
fn produce_repetitions(
    time: Res<SongTime>,
    In(responses): In<[Response; 256]>,
    repeaters: Query<&Repeater>,
    sheets: Query<(
        &SheetPosition,
        &Instance<Repeater>,
    )>,
)
    -> [(Response, Repetition); 256]
{
    responses.map(|out| (out, Repetition::new(out.seek_time))).tap_mut(|outputs| {
        sheets
            .iter()
            .filter(|(pos, _)| f32::EPSILON < pos.duration.raw())
            .filter(|(pos, _)| pos.scheduled_at(**time))
            .map(|(pos, instance)| (pos, repeaters.get(**instance).unwrap()))
            .filter(|(_, Repeater { period, .. })| f32::EPSILON < period.raw())
            .for_each(|(pos, Repeater { ping_pong, period, floor, ceil })| {
                (&mut outputs[pos.coverage()])
                    .iter_mut()
                    .filter(|(response, _)| pos.scheduled_at(response.seek_time))
                    .for_each(|(Response { seek_time, .. }, repetition)| {
                        let relative_time = *seek_time - pos.start;
                        let remainder_time = relative_time % period;
                        let division = (relative_time / period).floor();
                        let parity = division.raw() as i32 % 2;
                        let clamp_time = t32(((division * period) / pos.duration).raw());

                        *repetition = Repetition {
                            upper_clamp: ceil.eval(clamp_time),
                            lower_clamp: floor.eval(clamp_time),
                            time: pos.start + if *ping_pong && parity == 1 {
                                *period - remainder_time
                            } else {
                                remainder_time
                            }
                        }
                    })
            })
    })
}
