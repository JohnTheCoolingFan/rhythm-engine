use super::automation::Weight;
use crate::{sheet::*, utils::*, *};

use noisy_float::prelude::*;

pub struct RepeaterClamp {
    start: T64,
    end: T64,
    weight: Weight,
}

impl RepeaterClamp {
    pub fn eval(&self, t: T64) -> T64 {
        self.start.lerp(&self.end, self.weight.eval(t))
    }
}

#[derive(Component)]
pub struct Repeater {
    period: P64,
    ping_pong: bool,
    ceil: RepeaterClamp,
    floor: RepeaterClamp,
}

#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub struct ClampedTime {
    pub offset: P64,
    pub lower_clamp: T64,
    pub upper_clamp: T64,
}

impl ClampedTime {
    #[rustfmt::skip]
    pub fn new(offset: P64) -> Self {
        Self { offset, upper_clamp: t64(1.), lower_clamp: t64(0.) }
    }
}

#[derive(Component, Clone, Copy)]
pub struct RepeaterAffinity;

#[rustfmt::skip]
pub fn produce_repetitions(
    repeaters: Query<(&Sheet, &Repeater)>,
    mut time_tables: ResMut<TimeTables>,
) {
    let TimeTables { song_time, seek_times, clamped_times, .. } = &mut *time_tables;
    **clamped_times = seek_times.map(|seek_time| ClampedTime::new(seek_time));

    repeaters
        .iter()
        .filter(|(sheet, _)| f64::EPSILON < sheet.duration.raw())
        .filter(|(_, Repeater { period, .. })| f64::EPSILON < period.raw())
        .for_each(|(sheet, Repeater { ping_pong, period, floor, ceil })| {
            sheet.coverage().for_each(|index| {
                if let Some(time) = [seek_times[index], *song_time]
                    .iter()
                    .find(|time| sheet.scheduled_at(**time))
                {
                    let relative_time = *time - sheet.start;
                    let remainder_time = relative_time % period;
                    let division = (relative_time / period).floor();
                    let parity = division.raw() as i64 % 2;
                    let clamp_time = t64(((division * period) / sheet.duration).raw());

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
