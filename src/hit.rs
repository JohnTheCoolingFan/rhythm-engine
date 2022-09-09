use crate::{sheet::*, utils::*, *};

use bevy::prelude::*;
use derive_more::From;
use noisy_float::prelude::*;
use tap::tap::Tap;

enum PressKind {
    Press(N32),
    Hold(N32, N32),
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
    pub object_time: P32,
    pub hit_time: P32,
    pub layer: u8,
}

#[derive(Deref, DerefMut, From)]
pub struct HitRegister(pub [Option<HitInfo>; 4]);

impl Default for HitRegister {
    fn default() -> Self {
        HitRegister([None; 4])
    }
}

#[derive(Component)]
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

#[derive(Debug, PartialEq, Component)]
pub enum ResponseState {
    None,
    Hit(P32),
    Delegated(bool),
}

#[derive(Default, From, Deref, DerefMut, Clone, Copy)]
pub struct Delegated(pub bool);

fn clear_hit_responses(mut instances: Query<(&Sheet, &mut ResponseState)>) {
    instances
        .iter_mut()
        .for_each(|(_, mut response_state)| *response_state = ResponseState::None);
}

#[rustfmt::skip]
fn respond_to_hits(
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
        .filter(|(sheet, ..)| f32::EPSILON < sheet.duration.raw())
        .filter(|(sheet, ..)| sheet.scheduled_at(song_time))
        .for_each(|(sheet, Response { kind, layer }, mut state)| {
            use ResponseKind::*;
            use ResponseState::*;

            hits.iter()
                .flatten()
                .filter(|hit| sheet.scheduled_at(hit.hit_time) && hit.layer == *layer)
                .for_each(|hit| match (kind, &mut *state) {
                    (Commence | Switch, state) => *state = Delegated(true),
                    (Toggle, Delegated(delegate)) => *delegate = !*delegate,
                    (Toggle, state) => *state = Delegated(true),
                    (Follow(_), last_hit) => *last_hit = Hit(hit.object_time),
                    _ => {}
                });

            let adjusted_offset = match (kind, &*state) {
                (Commence, Delegated(delegate)) if !delegate => sheet.start,
                (Follow(ex), &Hit(hit)) if !(hit..hit + ex).contains(&song_time) => hit + ex,
                _ => song_time
            };

            let delegation = match (kind, &mut *state) {
                (Switch | Toggle, Delegated(state)) => *state,
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

    #[test]
    #[rustfmt::skip]
    fn hit_layers_and_scheduling() {
        let mut game = App::new();
        game.add_system(respond_to_hits);
        game.insert_resource(TimeTables { song_time: p32(300.), ..Default::default() });

        game.world.spawn().insert_bundle((
            Sheet { start: p32(0.), duration:  p32(1000.), coverage: Coverage(0, 0) },
            Response { kind: ResponseKind::Commence, layer: 0 },
            ResponseState::None
        ));

        // Wrong layer
        game.insert_resource(HitRegister([None, None, None, Some(HitInfo {
            object_time: p32(200.),
            hit_time: p32(300.),
            layer: 3,
        })]));

        game.update();
        assert_eq!(
            ResponseState::None,
            *game.world.query::<&ResponseState>().single(&game.world)
        );

        // Wrong scheduling
        game.insert_resource(HitRegister([None, None, None, Some(HitInfo {
            object_time: p32(200.),
            hit_time: p32(1100.),
            layer: 0,
        })]));

        game.update();
        assert_eq!(
            ResponseState::None,
            *game.world.query::<&ResponseState>().single(&game.world)
        );

        // Correct layer correct scheduling
        game.insert_resource(HitRegister([None, None, None, Some(HitInfo {
            object_time: p32(200.),
            hit_time: p32(300.),
            layer: 0,
        })]));

        game.update();
        assert_eq!(
            ResponseState::Delegated(true),
            *game.world.query::<&ResponseState>().single(&game.world)
        );
    }
}
