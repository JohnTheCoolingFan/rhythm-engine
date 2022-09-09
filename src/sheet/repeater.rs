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
    period: P32,
    ping_pong: bool,
    ceil: RepeaterClamp,
    floor: RepeaterClamp,
}

#[derive(Clone, Copy)]
pub struct ClampedTime {
    pub offset: P32,
    pub lower_clamp: T32,
    pub upper_clamp: T32,
}

impl ClampedTime {
    #[rustfmt::skip]
    pub fn new(offset: P32) -> Self {
        Self { offset, upper_clamp: t32(1.), lower_clamp: t32(0.) }
    }
}

#[derive(Component, Clone, Copy)]
pub struct RepeaterAffinity;

#[rustfmt::skip]
fn produce_repetitions(
    repeaters: Query<(&Sheet, &Repeater)>,
    mut time_tables: ResMut<TimeTables>,
) {
    let TimeTables { song_time, seek_times, clamped_times, .. } = &mut *time_tables;
    **clamped_times = seek_times.map(|seek_time| ClampedTime::new(seek_time));

    repeaters
        .iter()
        .filter(|(sheet, _)| f32::EPSILON < sheet.duration.raw())
        .filter(|(_, Repeater { period, .. })| f32::EPSILON < period.raw())
        .for_each(|(sheet, Repeater { ping_pong, period, floor, ceil })| {
            sheet.coverage().for_each(|index| {
                if let Some(time) = [seek_times[index], *song_time]
                    .iter()
                    .find(|time| sheet.scheduled_at(**time))
                {
                    let relative_time = *time - sheet.start;
                    let remainder_time = relative_time % period;
                    let division = (relative_time / period).floor();
                    let parity = division.raw() as i32 % 2;
                    let clamp_time = t32(((division * period) / sheet.duration).raw());

                    clamped_times[index] = ClampedTime {
                        upper_clamp: ceil.eval(clamp_time),
                        lower_clamp: floor.eval(clamp_time),
                        offset: sheet.start + if *ping_pong && parity == 1 {
                            *period - remainder_time
                        } else {
                            remainder_time
                        }
                    }
                }
            })
        })
}
