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

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use noisy_float::prelude::*;
use tap::TapOptional;

pub enum Modulation {
    Invalid,
    RGBA([T32; 4]),
    Luminosity(T32),
    Rotation(R32),
    Scale(R32),
    Translation(Vec2),
}

impl From<Vec2> for Modulation {
    fn from(point: Vec2) -> Self {
        Self::Translation(point)
    }
}

impl From<RGBA> for Modulation {
    fn from(color: RGBA) -> Self {
        Self::RGBA(*color)
    }
}

impl From<Luminosity> for Modulation {
    fn from(luminosity: Luminosity) -> Self {
        Self::Luminosity(*luminosity)
    }
}

impl From<Scale> for Modulation {
    fn from(scale: Scale) -> Self {
        Self::Scale(*scale)
    }
}

impl From<Rotation> for Modulation {
    fn from(theta: Rotation) -> Self {
        Self::Rotation(*theta)
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
    fn play(&self, channel: usize,  t: T32) -> Option<Modulation> {
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
    fn play(&self, channel: usize, t: T32) -> Option<Modulation> {
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
    colors: Ensemble<'w, 's, RGBA>,
    luminosities: Ensemble<'w, 's, Luminosity>,
    scales: Ensemble<'w, 's, Scale>,
    rotations: Ensemble<'w, 's, Rotation>,
}

#[rustfmt::skip]
pub fn harmonize(
    mut modulations: ResMut<Table<Option<Modulation>>>,
    seek_times: Res<Table<SeekTime>>,
    clamped_times: Res<Table<ClampedTime>>,
    delegations: Res<Table<Delegated>>,
    performers: Performers,
    automation_sources: Query<&Automation<T32>>,
    automations: Query<(
        &TemporalOffsets,
        &ChannelCoverage,
        &Sources<Automation<T32>>,
    )>,
) {

    modulations.fill_with(|| None);

    // First produce modulations with overlapping arrangements consisting of a
    //  - Primary sequence
    //  - Secondary sequence
    //  - Automation

    // Automations have to be arranged seperately because their offset has to be shifted
    // And because they do not have primary and secondary smenatics like sequences
    automations.iter().for_each(|(offsets, coverage, automation)| {
        coverage.iter().for_each(|index| {
            if let Some(t) = [clamped_times[index], ClampedTime::new(*seek_times[index])]
                .iter_mut()
                .find(|clamped_time| offsets.playable_at(clamped_time.offset))
                .tap_some_mut(|clamped_time| clamped_time.offset -= offsets.start)
                .and_then(|time| automation_sources
                    .get(*automation.pick(*delegations[index]))
                    .map(|automation| automation.play(*time))
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
                    .unwrap_or(Some(Modulation::Invalid))
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
}

pub struct HarmonizerPlugin;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum HarmonizerSet {
    PreArrange,
    Arrange,
    PostArrange,
}

#[rustfmt::skip]
impl Plugin for HarmonizerPlugin {
    fn build(&self, game: &mut App) {
        use HarmonizerSet::*;

        game.init_resource::<SongTime>()
            .init_resource::<Table<SeekTime>>()
            .init_resource::<Table<ClampedTime>>()
            .init_resource::<Table<Delegated>>()
            .init_resource::<HitRegister>()
            .init_resource::<SequenceArrangements<Spline>>()
            .init_resource::<SequenceArrangements<RGBA>>()
            .init_resource::<SequenceArrangements<Luminosity>>()
            .init_resource::<SequenceArrangements<Scale>>()
            .init_resource::<SequenceArrangements<Rotation>>()
            .init_resource::<Table<Option<Modulation>>>()
            .configure_sets((PreArrange, Arrange, PostArrange).chain())
            .add_systems((respond_to_hits, produce_repetitions)
                .chain()
                .in_set(PreArrange)
                .distributive_run_if(map_selected)
            )
            .add_systems((
                    arrange_sequences::<Spline>,
                    arrange_sequences::<RGBA>,
                    arrange_sequences::<Luminosity>,
                    arrange_sequences::<Scale>,
                    arrange_sequences::<Rotation>
                )
                .in_set(Arrange)
                .distributive_run_if(map_selected)
            )
            .add_system(harmonize
                .in_set(PostArrange)
                .run_if(map_selected)
            );
    }
}
