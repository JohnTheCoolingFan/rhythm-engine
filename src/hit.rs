use crate::{harmonizer::arranger::*, timing::*, utils::*};
use bevy::prelude::*;

use derive_more::From;
use noisy_float::prelude::*;

enum PressKind {
    Press(P32),
    Hold(P32, P32),
}

#[repr(u8)]
enum PressStrength {
    Single = 1,
    Double = 2,
    Triple = 3,
}

pub struct HitPrompt {
    offsets: TemporalOffsets,
    press_kind: PressKind,
    press_strength: PressStrength,
    press_phat_key: bool,
    signal_layer: u8,
}

#[derive(Clone, Copy)]
pub struct HitInfo {
    /// Object time is used instead of hit time to keep animations synced with music
    pub object_time: P32,
    pub hit_time: P32,
    pub layer: u8,
}

#[derive(Default, Deref, DerefMut, From, Resource)]
pub struct HitRegister(pub [Option<HitInfo>; 4]);

#[derive(Debug, Component)]
pub enum ResponseKind {
    Nil,
    /// Stays at 0 state until hit, once hit which it will commece from the current time.
    Commence,
    /// Switches to a delegate automation permenantly.
    Switch,
    /// Switches to a delegate automation but will switch back to the original
    /// automation on another hit. This can be repeated indefinetly.
    Toggle,
    /// Will stay at 0 state with no hit, for each hit it will play the automation
    /// from the hit time to hit time + excess.
    Follow(P32),
}

#[derive(Component)]
pub struct Response {
    pub kind: ResponseKind,
    pub layer: u8,
}

#[derive(Debug, PartialEq, Eq, Component)]
pub enum ResponseState {
    None,
    Hit(P32),
    Active(bool),
}

#[derive(Default, Debug, PartialEq, Eq, From, Deref, DerefMut, Clone, Copy)]
pub struct Delegated(pub bool);

#[rustfmt::skip]
pub fn respond_to_hits(
    hits: Res<HitRegister>,
    song_time: Res<SongTime>,
    mut seek_times: ResMut<Table<SeekTime>>,
    mut delegations: ResMut<Table<Delegated>>,
    mut responses: Query<(
        &TemporalOffsets,
        &ChannelCoverage,
        &Response,
        &mut ResponseState
    )>,
) {
    seek_times.fill_with(|| SeekTime(**song_time));
    delegations.fill_with(|| Delegated(false));

    responses
        .iter_mut()
        .filter(|(offsets, ..)| offsets.playable_at(**song_time))
        .for_each(|(offsets, coverage, Response { kind, layer }, mut state)| {
            use ResponseKind::*;
            use ResponseState::*;

            hits.iter()
                .flatten()
                .filter(|hit| offsets.scheduled_at(hit.hit_time) && hit.layer == *layer)
                .for_each(|hit| match (kind, &mut *state) {
                    (Toggle, Active(active)) => *active = !*active,
                    (Follow(_), last_hit) => *last_hit = Hit(hit.object_time),
                    (Commence | Switch | Toggle, state) => *state = Active(true),
                    _ => {}
                });

            let adjusted_offset = match (kind, &*state) {
                (Commence, Active(active)) if !active => offsets.start,
                (Follow(ex), &Hit(hit)) if !(hit..hit + ex).contains(&**song_time) => hit + ex,
                _ => **song_time
            };

            let delegation = match (kind, &mut *state) {
                (Switch | Toggle, Active(active)) => *active,
                _ => false
            };

            coverage.iter().for_each(|index| {
                *seek_times[index] = adjusted_offset;
                *delegations[index] = delegation;
            })
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use test_case::test_case;

    #[rustfmt::skip]
    #[test_case(300., 3, ResponseState::None; "wrong layer")]
    #[test_case(1100., 0, ResponseState::None; "wrong scheduling")]
    #[test_case(300., 0, ResponseState::Active(true); "correct layer and scheduling")]
    fn hit_layers_and_scheduling(time: f32, layer: u8, expected: ResponseState) {
        let mut game = App::new();
        game.add_system(respond_to_hits);

        game.insert_resource(SongTime(p32(time)))
            .insert_resource(Table::<SeekTime>::default())
            .insert_resource(Table::<ClampedTime>::default())
            .insert_resource(Table::<Delegated>::default());

        game.world.spawn((
            ResponseState::None,
            Response { kind: ResponseKind::Commence, layer: 0 },
            ChannelCoverage(vec![CoverageRange::new(0, 0)].into()),
            TemporalOffsets { start: p32(0.), duration:  p32(1000.) }
        ));

        game.insert_resource(HitRegister([
            None,
            None,
            None,
            Some(HitInfo {
                object_time: p32(time),
                hit_time: p32(time),
                layer,
            })
        ]));

        game.update();
        assert_eq!(expected, *game.world.query::<&ResponseState>().single(&game.world))
    }

    #[test]
    #[rustfmt::skip]
    fn hit_logic() {
        let mut game = App::new();
        game.add_system(respond_to_hits);
        game.world.spawn_batch([
            (
                ResponseState::None,
                Response { kind: ResponseKind::Commence, layer: 0 },
                ChannelCoverage(vec![CoverageRange::new(0, 0)].into()),
                TemporalOffsets { start: p32(0.), duration:  p32(400.) },
            ),
            (
                ResponseState::None,
                Response { kind: ResponseKind::Switch, layer: 0 },
                ChannelCoverage(vec![CoverageRange::new(1, 1)].into()),
                TemporalOffsets { start: p32(0.), duration:  p32(400.) },
            ),
            (
                ResponseState::None,
                Response { kind: ResponseKind::Toggle, layer: 0 },
                ChannelCoverage(vec![CoverageRange::new(2, 2)].into()),
                TemporalOffsets { start: p32(0.), duration:  p32(400.) },
            ),
            (
                ResponseState::None,
                Response { kind: ResponseKind::Follow(p32(50.)), layer: 0 },
                ChannelCoverage(vec![CoverageRange::new(3, 3)].into()),
                TemporalOffsets { start: p32(0.), duration:  p32(400.) },
            ),
        ]);

        // First hit
        game.insert_resource(SongTime(p32(100.)))
            .insert_resource(Table::<SeekTime>::default())
            .insert_resource(Table::<ClampedTime>::default())
            .insert_resource(Table::<Delegated>::default());

        game.insert_resource(HitRegister([None, None, None, Some(HitInfo {
            object_time: p32(100.),
            hit_time: p32(100.),
            layer: 0,
        })]));

        game.update();
        game.world.query::<&ResponseState>().iter(&game.world).for_each(|state| {
            assert_ne!(ResponseState::None, *state)
        });
        assert_eq!(
            game.world.resource::<Table<SeekTime>>()[..4],
            [100.; 4].map(|t| SeekTime(p32(t)))
        );
        assert_eq!(
            game.world.resource::<Table<Delegated>>()[..4],
            [false, true, true, false].map(Delegated)
        );

        // State after first hit and before second
        game.insert_resource(SongTime(p32(200.)))
            .insert_resource(Table::<SeekTime>::default())
            .insert_resource(Table::<ClampedTime>::default())
            .insert_resource(Table::<Delegated>::default());

        game.insert_resource(HitRegister([None, None, None, None]));

        game.update();
        assert_eq!(
            game.world.resource::<Table<SeekTime>>()[..4],
            [200., 200., 200., 150.].map(|t| SeekTime(p32(t)))
        );

        // Second hit
        game.insert_resource(SongTime(p32(300.)))
            .insert_resource(Table::<SeekTime>::default())
            .insert_resource(Table::<ClampedTime>::default())
            .insert_resource(Table::<Delegated>::default());

        game.insert_resource(HitRegister([None, None, None, Some(HitInfo {
            object_time: p32(300.),
            hit_time: p32(300.),
            layer: 0,
        })]));

        game.update();
        assert_eq!(
            game.world.resource::<Table<SeekTime>>()[..4],
            [300., 300., 300., 300.].map(|t| SeekTime(p32(t)))
        );
        assert_eq!(
            game.world.resource::<Table<Delegated>>()[..4],
            [false, true, false, false].map(Delegated)
        );
    }
}
