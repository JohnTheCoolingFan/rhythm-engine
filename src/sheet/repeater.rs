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

#[derive(Component)]
pub struct RepeaterAffinity(bool);

#[derive(Clone, Copy)]
pub struct RepeaterOutput {
    repeat_time: P32,
    lower_clamp: T32,
    upper_clamp: T32,
}

impl RepeaterOutput {
    fn new(seek_time: P32) -> Self {
        Self {
            repeat_time: seek_time,
            lower_clamp: t32(0.),
            upper_clamp: t32(1.),
        }
    }
}

#[rustfmt::skip]
fn eval_repeaters(
    time: Res<SongTime>,
    In(response_outputs): In<[ResponseOutput; 256]>,
    repeaters: Query<&Repeater>,
    sheets: Query<(
        &SheetPosition,
        &Instance<Repeater>,
    )>,
)
    -> [(ResponseOutput, RepeaterOutput); 256]
{
    response_outputs.map(|out| (out, RepeaterOutput::new(out.seek_time))).tap_mut(|outputs| {
        sheets
            .iter()
            .filter(|(sheet, _)| f32::EPSILON < sheet.duration.raw())
            .filter(|(sheet, _)| sheet.scheduled_at(**time))
            .map(|(sheet, instance)| (sheet, repeaters.get(instance.entity).unwrap()))
            .filter(|(_, Repeater { period, .. })| f32::EPSILON < period.raw())
            .for_each(|(sheet, Repeater { ping_pong, period, floor, ceil })| {
                (&mut outputs[sheet.coverage()])
                    .iter_mut()
                    .filter(|(response_output, _)| sheet.scheduled_at(response_output.seek_time))
                    .for_each(|(ResponseOutput { seek_time, .. }, repeater_output)| {
                        let relative_time = *seek_time - sheet.start;
                        let remainder_time = relative_time % period;
                        let division = (relative_time / period).floor();
                        let parity = division.raw() as i32 % 2;
                        let clamp_time = t32(((division * period) / sheet.duration).raw());

                        *repeater_output = RepeaterOutput {
                            upper_clamp: ceil.eval(clamp_time),
                            lower_clamp: floor.eval(clamp_time),
                            repeat_time: sheet.start + if *ping_pong && parity == 1 {
                                *period - remainder_time
                            } else {
                                remainder_time
                            }
                        }
                    })
            })
    });

    todo!()
}
