use super::*;
use crate::{automation::*, utils::*};

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

impl Repeater {
    fn new(period: P64) -> Self {
        Repeater {
            period,
            ping_pong: false,
            ceil: RepeaterClamp {
                start: t64(0.),
                end: t64(0.),
                weight: Weight::Quadratic(r64(0.)),
            },
            floor: RepeaterClamp {
                start: t64(1.),
                end: t64(1.),
                weight: Weight::Quadratic(r64(0.)),
            },
        }
    }
}

#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub struct ClampedTime {
    pub offset: P64,
    pub upper_clamp: T64,
    pub lower_clamp: T64,
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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use test_case::test_case;

    fn sheet() -> Sheet {
        Sheet {
            start: p64(0.),
            duration: p64(1000.),
            coverage: Coverage(0, 0),
        }
    }

    #[rustfmt::skip]
    #[test_case(
        sheet(),
        Repeater::new(p64(500.)),
        &[(p64(250.), ClampedTime::new(p64(500.)))];
        "test"
    )]
    fn repetition_logic(sheet: Sheet, repeater: Repeater, co_vals: &[(P64, ClampedTime)]) {
        let mut game = App::new();
        game.add_system(produce_repetitions);
        game.world.spawn().insert_bundle((sheet.clone(), repeater));

        co_vals.iter().for_each(|(time, expected)| {
            game.insert_resource(TimeTables { song_time: *time, ..Default::default() });
            game.update();
            game.world
                .resource::<TimeTables>()
                .clamped_times[sheet.coverage()]
                .iter()
                .for_each(|clamped_time| assert_eq!(expected, clamped_time));
        });
    }
}
