pub mod arranger;
pub mod repeater;

use arranger::*;
use repeater::*;

use crate::{
    automation::{sequence::*, spline::*, *},
    hit::*,
    map_selected,
    timing::*,
    utils::*,
};

use core::iter::once as iter_once;

use bevy::prelude::*;
use bevy::{ecs::system::SystemParam, math::DVec2};
use bevy_system_graph::*;
use noisy_float::prelude::*;
use tap::{Conv, Tap, TapOptional};

pub enum Modulation {
    None,
    Rgba([T64; 4]),
    Luminosity(T64),
    Rotation { theta: R64, ctrl: Option<DVec2> },
    Scale { magnitude: R64, ctrl: Option<DVec2> },
    Position { shift: DVec2, start: Option<DVec2> },
}

impl From<DVec2> for Modulation {
    fn from(point: DVec2) -> Self {
        Self::Position {
            shift: point,
            start: None,
        }
    }
}

impl From<Rgba> for Modulation {
    fn from(color: Rgba) -> Self {
        Self::Rgba(*color)
    }
}

impl From<Luminosity> for Modulation {
    fn from(luminosity: Luminosity) -> Self {
        Self::Luminosity(*luminosity)
    }
}

impl From<Scale> for Modulation {
    fn from(scale: Scale) -> Self {
        Self::Scale {
            magnitude: *scale,
            ctrl: None,
        }
    }
}

impl From<Rotation> for Modulation {
    fn from(theta: Rotation) -> Self {
        Self::Rotation {
            theta: *theta,
            ctrl: None,
        }
    }
}

#[derive(SystemParam)]
pub struct Ensemble<'w, 's, T: Component + Default> {
    sources: Query<'w, 's, &'static Sequence<T>>,
    arrangements: Res<'w, SequenceArrangements<T>>,
}

impl<'w, 's, T: Component + Default> Ensemble<'w, 's, T> {
    #[rustfmt::skip]
    fn get(&self, channel: usize) -> Option<Arrangement<&Sequence<T>>> {
        self.arrangements[channel].as_ref().map(|arrangement| Arrangement {
            offset: arrangement.offset,
            primary: self.sources.get(*arrangement.primary).unwrap(),
            secondary: arrangement.secondary.map(|secondary| self
                .sources
                .get(*secondary)
                .unwrap()
            ),
        })
    }
}

impl<'w, 's> Ensemble<'w, 's, Spline> {
    #[rustfmt::skip]
    fn play(&self, channel: usize,  t: T64) -> Option<Modulation> {
        self.get(channel).and_then(|arrangement| arrangement
            .secondary
            .is_none()
            .then(|| arrangement.primary.play(t, arrangement.offset).into())
        )
    }
}

#[rustfmt::skip]
impl<'w, 's, T> Ensemble<'w, 's, T>
where
    T: Default + Component + Clone + Copy + Lerp<Output = T>,
    Modulation: From<T>,
{
    fn play(&self, channel: usize, t: T64) -> Option<Modulation> {
        self.get(channel).and_then(|arrangement| arrangement
            .secondary
            .map(|secondary| arrangement
                .primary
                .play(arrangement.offset)
                .lerp(&secondary.play(arrangement.offset), t)
                .into()
            )
        )
    }

    fn play_primary(&self, channel: usize) -> Option<Modulation> {
        self.get(channel).and_then(|arrangement| arrangement
            .secondary
            .is_none()
            .then(|| arrangement.primary.play(arrangement.offset).into())
        )
    }
}

#[derive(SystemParam)]
pub struct Performers<'w, 's> {
    splines: Ensemble<'w, 's, Spline>,
    colors: Ensemble<'w, 's, Rgba>,
    luminosities: Ensemble<'w, 's, Luminosity>,
    scales: Ensemble<'w, 's, Scale>,
    rotations: Ensemble<'w, 's, Rotation>,
}

#[rustfmt::skip]
pub fn harmonize(
    mut modulations: ResMut<Table<Option<Modulation>>>,
    time_tables: ResMut<TimeTables>,
    performers: Performers,
    geom_ctrl_sources: Query<&GeometryCtrl>,
    automation_sources: Query<&Automation<T64>>,
    geom_ctrls: Query<(
        &Sheet,
        &GenID<GeometryCtrl>
    )>,
    // Automations have to be arranged seperately because their offset has to be shifted
    // And because they do not have primary and secondary smenatics like sequences
    automations: Query<(
        &Sheet,
        &Sources<Automation<T64>>,
        Option<&RepeaterAffinity>
    )>,
) {
    let TimeTables { song_time, seek_times, clamped_times, delegations } = *time_tables;

    modulations.fill_with(|| None);

    // First produce modulations with overlapping arrangements consisting of a
    //  - Primary sequence
    //  - Secondary sequence
    //  - Automation
    automations.iter().for_each(|(sheet, automation, affinity)| {
        sheet.coverage().for_each(|index| {
            if let Some(t) = affinity
                .map(|_| clamped_times[index])
                .into_iter()
                .chain(iter_once(ClampedTime::new(seek_times[index])))
                .find(|ClampedTime { offset, .. }| sheet.playable_at(*offset))
                .tap_some_mut(|clamped_time| clamped_time.offset -= sheet.start)
                .and_then(|time| automation_sources
                    .get(*automation.pick(*delegations[index]))
                    .map(|automation| automation.play(time))
                    .ok()
                )
            {
                let performances = [
                    performers.splines.play(index, t),
                    performers.colors.play(index, t),
                    performers.luminosities.play(index, t),
                    performers.scales.play(index, t),
                    performers.rotations.play(index, t),
                ];

                modulations[index] = performances
                    .into_iter()
                    .find(Option::is_some)
                    .unwrap_or(Some(Modulation::None))
            }
        })
    });

    // Then produce modulations with only a Primary sequence
    modulations
        .iter_mut()
        .enumerate()
        .filter(|(_, modulation)|  modulation.is_none())
        .for_each(|(index, modulation)| {
            let performances = [
                performers.colors.play_primary(index),
                performers.luminosities.play_primary(index),
                performers.scales.play_primary(index),
                performers.rotations.play_primary(index),
            ];

            *modulation = performances
                .into_iter()
                .find(Option::is_some)
                .flatten()
        });

    // Finaly add geometry control information to modulations that can be controlled by points
    geom_ctrls.iter().filter(|(sheet, ..)| sheet.playable_at(song_time)).for_each(|(sheet, genid)| {
        modulations[sheet.coverage()].iter_mut().flatten().for_each(|modulation| {
            if let
                | Modulation::Scale { ctrl: point, .. }
                | Modulation::Rotation { ctrl: point, .. }
                | Modulation::Position { start: point, .. }
                = modulation
            {
                *point = geom_ctrl_sources.get(**genid).ok().map(|ctrl| **ctrl)
            }
        })
    })
}

pub struct HarmonizerPlugin;

#[rustfmt::skip]
impl Plugin for HarmonizerPlugin {
    fn build(&self, game: &mut App) {
        game.init_resource::<TimeTables>()
            .init_resource::<HitRegister>()
            .init_resource::<SequenceArrangements<Spline>>()
            .init_resource::<SequenceArrangements<Rgba>>()
            .init_resource::<SequenceArrangements<Luminosity>>()
            .init_resource::<SequenceArrangements<Scale>>()
            .init_resource::<SequenceArrangements<Rotation>>()
            .init_resource::<Table<Option<Modulation>>>()
            .add_system_set(
                SystemGraph::new().tap(|sysg| {
                    sysg.root(respond_to_hits)
                        .then(repeater::produce_repetitions)
                        .fork((
                            arrange_sequences::<Spline>,
                            arrange_sequences::<Rgba>,
                            arrange_sequences::<Luminosity>,
                            arrange_sequences::<Scale>,
                            arrange_sequences::<Rotation>
                        ))
                        .join(harmonize);
                })
                .conv::<SystemSet>()
                .with_run_criteria(map_selected)
            );
    }
}
