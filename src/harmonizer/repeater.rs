use super::*;
use crate::{automation::*, utils::*};
use noisy_float::prelude::*;

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

impl Repeater {
    fn new(period: P32) -> Self {
        Repeater {
            period,
            ping_pong: false,
            ceil: RepeaterClamp {
                start: t32(1.),
                end: t32(1.),
                weight: Weight::Quadratic(r32(0.)),
            },
            floor: RepeaterClamp {
                start: t32(0.),
                end: t32(0.),
                weight: Weight::Quadratic(r32(0.)),
            },
        }
    }
}

#[rustfmt::skip]
pub fn produce_repetitions(
    repeaters: Query<(&TemporalOffsets, &ChannelCoverage, &Repeater)>,
    song_time: Res<SongTime>,
    seek_times: Res<Table<SeekTime>>,
    mut clamped_times: ResMut<Table<ClampedTime>>,
) {
    **clamped_times = seek_times.map(|t| ClampedTime::new(*t));

    repeaters
        .iter()
        .filter(|(offsets, ..)| f32::EPSILON < offsets.duration.raw())
        .filter(|(.., Repeater { period, .. })| f32::EPSILON < period.raw())
        .for_each(|(offsets, coverage, Repeater { ping_pong, period, floor, ceil })| {
            coverage.iter().for_each(|index| {
                if let Some(time) = [*seek_times[index], **song_time]
                    .iter()
                    .find(|time| offsets.scheduled_at(**time))
                {
                    let relative_time = *time - offsets.start;
                    let remainder_time = relative_time % period;
                    let division = (relative_time / period).floor();
                    let parity = division.raw() as i32 % 2;
                    let clamp_time = t32(((division * period) / offsets.duration).raw());

                    clamped_times[index] = ClampedTime {
                        upper_clamp: ceil.eval(clamp_time),
                        lower_clamp: floor.eval(clamp_time),
                        offset: offsets.start + if *ping_pong && parity == 1 {
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
    use tap::Pipe;
    use test_case::test_case;

    fn offset_coverage_bundle() -> (TemporalOffsets, ChannelCoverage) {
        (
            TemporalOffsets {
                start: p32(0.),
                duration: p32(2000.),
            },
            ChannelCoverage(vec![CoverageRange::new(0, 0)].into()),
        )
    }

    #[rustfmt::skip]
    #[test_case(
        offset_coverage_bundle(),
        Repeater::new(p32(500.)),
        p32(250.),
        ClampedTime::new(p32(250.));
        "simple division 0.5"
    )]
    #[test_case(
        offset_coverage_bundle(),
        Repeater::new(p32(500.)),
        p32(750.),
        ClampedTime::new(p32(250.));
        "simple division 1.5"
    )]
    #[test_case(
        offset_coverage_bundle(),
        Repeater::new(p32(500.)),
        p32(750.),
        ClampedTime::new(p32(250.));
        "simple division 2.5"
    )]
    #[test_case(
        offset_coverage_bundle(),
        Repeater {
            ceil: RepeaterClamp {
                weight: Weight::Quadratic(r32(0.)),
                start: t32(1.),
                end: t32(0.),
            },
            ..Repeater::new(p32(500.))
        },
        p32(250.),
        ClampedTime::new(p32(250.));
        "linear decreasing ceil division 0.5"
    )]
    #[test_case(
        offset_coverage_bundle(),
        Repeater {
            ceil: RepeaterClamp {
                weight: Weight::Quadratic(r32(0.)),
                start: t32(1.),
                end: t32(0.),
            },
            ..Repeater::new(p32(500.))
        },
        p32(750.),
        ClampedTime {
            upper_clamp: t32(0.75),
            lower_clamp: t32(0.0),
            ..ClampedTime::new(p32(250.))
        };
        "linear decreasing ceil division 1.5"
    )]
    #[test_case(
        offset_coverage_bundle(),
        Repeater { ping_pong: true, ..Repeater::new(p32(500.)) },
        p32(400.),
        ClampedTime::new(p32(400.));
        "pingpong division 0.5"
    )]
    #[test_case(
        offset_coverage_bundle(),
        Repeater { ping_pong: true, ..Repeater::new(p32(500.)) },
        p32(600.),
        ClampedTime::new(p32(400.));
        "pingpong division 1.5"
    )]
    #[test_case(
        offset_coverage_bundle(),
        Repeater { ping_pong: true, ..Repeater::new(p32(500.)) },
        p32(1100.),
        ClampedTime::new(p32(100.));
        "pingpong division 2.5"
    )]
    #[test_case(
        (
            TemporalOffsets { start: p32(1000.), ..offset_coverage_bundle().0 },
            offset_coverage_bundle().1
        ),
        Repeater::new(p32(500.)),
        p32(500.),
        ClampedTime::new(p32(500.));
        "shifted division -0.5"
    )]
    #[test_case(
        (
            TemporalOffsets { start: p32(1000.), ..offset_coverage_bundle().0 },
            offset_coverage_bundle().1
        ),
        Repeater::new(p32(500.)),
        p32(2000.),
        ClampedTime::new(p32(1000.));
        "shifted division 1."
    )]
    #[test_case(
        (
            TemporalOffsets { start: p32(1000.), ..offset_coverage_bundle().0 },
            offset_coverage_bundle().1
        ),
        Repeater::new(p32(500.)),
        p32(4000.),
        ClampedTime::new(p32(4000.));
        "shifted division overshoot 1."
    )]
    fn repetition_logic(
        (offsets, coverage): (TemporalOffsets, ChannelCoverage),
        repeater: Repeater,
        time: P32,
        expected: ClampedTime
    ) {
        let mut game = App::new();

        game.init_resource::<HitRegister>()
            .insert_resource(SongTime(time))
            .init_resource::<Table<SeekTime>>()
            .init_resource::<Table<ClampedTime>>()
            .init_resource::<Table<Delegated>>()
            .add_systems((respond_to_hits, produce_repetitions).chain());

        game.world.spawn((offsets, coverage.clone(), repeater));

        game.update();

        game.world
            .resource::<Table<ClampedTime>>()
            .pipe_ref(|clamped_times| coverage.iter().map(|index| clamped_times[index]))
            .for_each(|clamped_time| assert_eq!(expected, clamped_time));
    }
}
