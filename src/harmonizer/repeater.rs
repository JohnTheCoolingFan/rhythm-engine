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
                start: t64(1.),
                end: t64(1.),
                weight: Weight::Quadratic(r64(0.)),
            },
            floor: RepeaterClamp {
                start: t64(0.),
                end: t64(0.),
                weight: Weight::Quadratic(r64(0.)),
            },
        }
    }
}

#[rustfmt::skip]
pub fn produce_repetitions(
    repeaters: Query<(&Sheet, &Repeater)>,
    mut time_tables: ResMut<TimeTables>,
) {
    let TimeTables { song_time, seek_times, clamped_times, .. } = &mut *time_tables;
    **clamped_times = seek_times.map(ClampedTime::new);

    repeaters
        .iter()
        .filter(|(sheet, _)| f64::EPSILON < sheet.offsets.duration.raw())
        .filter(|(_, Repeater { period, .. })| f64::EPSILON < period.raw())
        .for_each(|(sheet, Repeater { ping_pong, period, floor, ceil })| {
            sheet.coverage().for_each(|index| {
                if let Some(time) = [seek_times[index], *song_time]
                    .iter()
                    .find(|time| sheet.offsets.scheduled_at(**time))
                {
                    let relative_time = *time - sheet.offsets.start;
                    let remainder_time = relative_time % period;
                    let division = (relative_time / period).floor();
                    let parity = division.raw() as i64 % 2;
                    let clamp_time = t64(((division * period) / sheet.offsets.duration).raw());

                    clamped_times[index] = ClampedTime {
                        upper_clamp: ceil.eval(clamp_time),
                        lower_clamp: floor.eval(clamp_time),
                        offset: sheet.offsets.start + if *ping_pong && parity == 1 {
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
    use bevy_system_graph::*;
    use pretty_assertions::assert_eq;
    use tap::Pipe;
    use test_case::test_case;

    fn sheet() -> Sheet {
        Sheet {
            coverage: vec![CoverageRange::new(0, 0)].into(),
            offsets: TemporalOffsets {
                start: p64(0.),
                duration: p64(2000.),
            },
        }
    }

    #[rustfmt::skip]
    #[test_case(
        sheet(),
        Repeater::new(p64(500.)),
        p64(250.),
        ClampedTime::new(p64(250.));
        "simple division 0.5"
    )]
    #[test_case(
        sheet(),
        Repeater::new(p64(500.)),
        p64(750.),
        ClampedTime::new(p64(250.));
        "simple division 1.5"
    )]
    #[test_case(
        sheet(),
        Repeater::new(p64(500.)),
        p64(750.),
        ClampedTime::new(p64(250.));
        "simple division 2.5"
    )]
    #[test_case(
        sheet(),
        Repeater {
            ceil: RepeaterClamp {
                weight: Weight::Quadratic(r64(0.)),
                start: t64(1.),
                end: t64(0.),
            },
            ..Repeater::new(p64(500.))
        },
        p64(250.),
        ClampedTime::new(p64(250.));
        "linear decreasing ceil division 0.5"
    )]
    #[test_case(
        sheet(),
        Repeater {
            ceil: RepeaterClamp {
                weight: Weight::Quadratic(r64(0.)),
                start: t64(1.),
                end: t64(0.),
            },
            ..Repeater::new(p64(500.))
        },
        p64(750.),
        ClampedTime {
            upper_clamp: t64(0.75),
            lower_clamp: t64(0.0),
            ..ClampedTime::new(p64(250.))
        };
        "linear decreasing ceil division 1.5"
    )]
    #[test_case(
        sheet(),
        Repeater { ping_pong: true, ..Repeater::new(p64(500.)) },
        p64(400.),
        ClampedTime::new(p64(400.));
        "pingpong division 0.5"
    )]
    #[test_case(
        sheet(),
        Repeater { ping_pong: true, ..Repeater::new(p64(500.)) },
        p64(600.),
        ClampedTime::new(p64(400.));
        "pingpong division 1.5"
    )]
    #[test_case(
        sheet(),
        Repeater { ping_pong: true, ..Repeater::new(p64(500.)) },
        p64(1100.),
        ClampedTime::new(p64(100.));
        "pingpong division 2.5"
    )]
    #[test_case(
        Sheet { offsets: TemporalOffsets { start: p64(1000.), ..sheet().offsets }, ..sheet() },
        Repeater::new(p64(500.)),
        p64(500.),
        ClampedTime::new(p64(500.));
        "shifted division -0.5"
    )]
    #[test_case(
        Sheet { offsets: TemporalOffsets { start: p64(1000.), ..sheet().offsets }, ..sheet() },
        Repeater::new(p64(500.)),
        p64(2000.),
        ClampedTime::new(p64(1000.));
        "shifted division 1."
    )]
    #[test_case(
        Sheet { offsets: TemporalOffsets { start: p64(1000.), ..sheet().offsets }, ..sheet() },
        Repeater::new(p64(500.)),
        p64(4000.),
        ClampedTime::new(p64(4000.));
        "shifted division overshoot 1."
    )]
    fn repetition_logic(sheet: Sheet, repeater: Repeater, time: P64, expected: ClampedTime) {
        let mut game = App::new();
        game.init_resource::<HitRegister>();
        game.insert_resource(TimeTables { song_time: time, ..Default::default() });
        game.add_system_set(
            SystemGraph::new().tap(|sysg| {
                sysg.root(respond_to_hits)
                    .then(produce_repetitions);
            })
            .conv::<SystemSet>()
        );

        game.world.spawn((sheet.clone(), repeater));
        game.update();

        game.world
            .resource::<TimeTables>()
            .clamped_times
            .pipe_ref(|clamped_times| sheet.coverage().map(|index| clamped_times[index]))
            .for_each(|clamped_time| assert_eq!(expected, clamped_time));
    }
}
