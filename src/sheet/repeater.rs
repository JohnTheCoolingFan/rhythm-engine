use super::automation::Weight;
use crate::{hit::*, sheet::*, utils::*, *};

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
    pub fn new(seek_time: P32) -> Self {
        Self {
            time: seek_time,
            lower_clamp: t32(0.),
            upper_clamp: t32(1.),
        }
    }
}

#[rustfmt::skip]
fn produce_repetitions(
    song_time: Res<SongTime>,
    mut seek_times: ResMut<Table<SeekTime>>,
    mut repetitions: ResMut<Table<Repetition>>,
    repeaters: Query<&Repeater>,
    gen_ids: Query<(
        &Sheet,
        &GenID<Repeater>,
    )>,
)
{
    gen_ids
        .iter()
        .filter(|(sheet, _)| f32::EPSILON < sheet.duration.raw())
        .map(|(sheet, gen_id)| (sheet, repeaters.get(**gen_id).unwrap()))
        .filter(|(_, Repeater { period, .. })| f32::EPSILON < period.raw())
        .for_each(|(sheet, Repeater { ping_pong, period, floor, ceil })| {
            sheet.coverage::<usize>().for_each(|index| {
                if let Some(time) = [*seek_times[index], **song_time]
                    .iter()
                    .find(|time| sheet.scheduled_at(**time))
                {
                    *seek_times[index] = *time;

                    let relative_time = *time - sheet.start;
                    let remainder_time = relative_time % period;
                    let division = (relative_time / period).floor();
                    let parity = division.raw() as i32 % 2;
                    let clamp_time = t32(((division * period) / sheet.duration).raw());

                    repetitions[index] = Repetition {
                        upper_clamp: ceil.eval(clamp_time),
                        lower_clamp: floor.eval(clamp_time),
                        time: sheet.start + if *ping_pong && parity == 1 {
                            *period - remainder_time
                        } else {
                            remainder_time
                        }
                    }
                }
            })
        })
}
