use crate::{harmonizer::arranger::*, utils::*};
use bevy::prelude::*;

use derive_more::From;
use noisy_float::prelude::*;

enum PressKind {
    Press(N64),
    Hold(N64, N64),
}

#[repr(u8)]
enum PressStrength {
    Single = 1,
    Double = 2,
    Triple = 3,
}

pub struct HitPrompt {
    press_kind: PressKind,
    press_strength: PressStrength,
    press_phat_key: bool,
    signal_layer: u8,
}

#[derive(Clone, Copy)]
pub struct HitInfo {
    /// Object time is used instead of hit time to keep animations synced with music
    pub object_time: P64,
    pub hit_time: P64,
    pub layer: u8,
}

#[derive(Deref, DerefMut, From)]
pub struct HitRegister(pub [Option<HitInfo>; 4]);

impl Default for HitRegister {
    fn default() -> Self {
        HitRegister([None; 4])
    }
}

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
    Follow(P64),
}

#[derive(Component)]
pub struct Response {
    pub kind: ResponseKind,
    pub layer: u8,
}

#[derive(Debug, PartialEq, Component)]
pub enum ResponseState {
    None,
    Hit(P64),
    Active(bool),
}

#[derive(Default, Debug, PartialEq, From, Deref, DerefMut, Clone, Copy)]
pub struct Delegated(pub bool);

pub fn clear_hit_responses(mut instances: Query<(&Sheet, &mut ResponseState)>) {
    instances
        .iter_mut()
        .for_each(|(_, mut response_state)| *response_state = ResponseState::None);
}

#[rustfmt::skip]
pub fn respond_to_hits(
    hits: Res<HitRegister>,
    mut time_tables: ResMut<TimeTables>,
    mut responses: Query<(
        &Sheet,
        &Response,
        &mut ResponseState
    )>,
) {
    let song_time = time_tables.song_time;

    time_tables.seek_times.fill_with(|| song_time);
    time_tables.delegations.fill_with(|| Delegated(false));

    responses
        .iter_mut()
        .filter(|(sheet, ..)| sheet.playable_at(song_time))
        .for_each(|(sheet, Response { kind, layer }, mut state)| {
            use ResponseKind::*;
            use ResponseState::*;

            hits.iter()
                .flatten()
                .filter(|hit| sheet.scheduled_at(hit.hit_time) && hit.layer == *layer)
                .for_each(|hit| match (kind, &mut *state) {
                    (Toggle, Active(active)) => *active = !*active,
                    (Follow(_), last_hit) => *last_hit = Hit(hit.object_time),
                    (Commence | Switch | Toggle, state) => *state = Active(true),
                    _ => {}
                });

            let adjusted_offset = match (kind, &*state) {
                (Commence, Active(active)) if !active => sheet.start,
                (Follow(ex), &Hit(hit)) if !(hit..hit + ex).contains(&song_time) => hit + ex,
                _ => song_time
            };

            let delegation = match (kind, &mut *state) {
                (Switch | Toggle, Active(active)) => *active,
                _ => false
            };

            sheet.coverage().for_each(|index| {
                time_tables.seek_times[index] = adjusted_offset;
                *time_tables.delegations[index] = delegation;
            })
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use test_case::test_case;

    #[rustfmt::skip]
    #[test_case(300., 3, ResponseState::None; "wrong Layer")]
    #[test_case(1100., 0, ResponseState::None; "wrong scheduling")]
    #[test_case(300., 0, ResponseState::Active(true); "correct layer and scheduling")]
    fn hit_layers_and_scheduling(time: f64, layer: u8, expected: ResponseState) {
        let mut game = App::new();
        game.add_system(respond_to_hits);
        game.insert_resource(TimeTables { song_time: p64(time), ..Default::default() });
        game.world.spawn().insert_bundle((
            Sheet { start: p64(0.), duration:  p64(1000.), coverage: Coverage(0, 0) },
            Response { kind: ResponseKind::Commence, layer: 0 },
            ResponseState::None
        ));

        game.insert_resource(HitRegister([
            None,
            None,
            None,
            Some(HitInfo {
                object_time: p64(time),
                hit_time: p64(time),
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
                Sheet { start: p64(0.), duration:  p64(400.), coverage: Coverage(0, 0) },
                Response { kind: ResponseKind::Commence, layer: 0 },
                ResponseState::None,
            ),
            (
                Sheet { start: p64(0.), duration:  p64(400.), coverage: Coverage(1, 1) },
                Response { kind: ResponseKind::Switch, layer: 0 },
                ResponseState::None,
            ),
            (
                Sheet { start: p64(0.), duration:  p64(400.), coverage: Coverage(2, 2) },
                Response { kind: ResponseKind::Toggle, layer: 0 },
                ResponseState::None,
            ),
            (
                Sheet { start: p64(0.), duration:  p64(400.), coverage: Coverage(3, 3) },
                Response { kind: ResponseKind::Follow(p64(50.)), layer: 0 },
                ResponseState::None,
            ),
        ]);

        // First hit
        game.insert_resource(TimeTables { song_time: p64(100.), ..Default::default() });
        game.insert_resource(HitRegister([None, None, None, Some(HitInfo {
            object_time: p64(100.),
            hit_time: p64(100.),
            layer: 0,
        })]));

        game.update();
        game.world.query::<&ResponseState>().iter(&game.world).for_each(|state| {
            assert_ne!(ResponseState::None, *state)
        });
        assert_eq!(
            game.world.resource::<TimeTables>().seek_times[..4],
            [100.; 4].map(p64)
        );
        assert_eq!(
            game.world.resource::<TimeTables>().delegations[..4],
            [false, true, true, false].map(Delegated)
        );

        // State after first hit and before second
        game.insert_resource(TimeTables { song_time: p64(200.), ..Default::default() });
        game.insert_resource(HitRegister([None, None, None, None]));

        game.update();
        assert_eq!(
            game.world.resource::<TimeTables>().seek_times[..4],
            [200., 200., 200., 150.].map(p64)
        );

        // Second hit
        game.insert_resource(TimeTables { song_time: p64(300.), ..Default::default() });
        game.insert_resource(HitRegister([None, None, None, Some(HitInfo {
            object_time: p64(300.),
            hit_time: p64(300.),
            layer: 0,
        })]));

        game.update();
        assert_eq!(
            game.world.resource::<TimeTables>().seek_times[..4],
            [300., 300., 300., 300.].map(p64)
        );
        assert_eq!(
            game.world.resource::<TimeTables>().delegations[..4],
            [false, true, false, false].map(Delegated)
        );
    }
}
