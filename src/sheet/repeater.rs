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
    mut repeater_sheets: Query<(
        &SheetPosition,
        &Instance<Repeater>,
    )>,
)
    -> [(ResponseOutput, RepeaterOutput); 256]
{
    response_outputs.map(|out| (out, RepeaterOutput::new(out.seek_time))).tap_mut(|outputs| {
        repeater_sheets
            .iter_mut()
            .filter(|(sheet, _)| sheet.scheduled_at(**time))
            .for_each(|(sheet, instance)| {
                let Repeater { ping_pong, period, floor, ceil } = repeaters
                    .get(instance.entity)
                    .unwrap();

            })
    });

    todo!()
}
