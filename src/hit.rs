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

#[derive(Component)]
pub enum ResponseState {
    Nil,
    Hit(P32),
    Delegated(bool),
}

#[derive(Default, From, Deref, DerefMut, Clone, Copy)]
pub struct Delegated(bool);

fn clear_hit_responses(mut instances: Query<(&Sheet, &mut ResponseState)>) {
    instances
        .iter_mut()
        .for_each(|(_, mut response_state)| *response_state = ResponseState::Nil);
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

    #[test]
    #[rustfmt::skip]
    fn hit_responses() {
        let mut app = App::new();

        /*app.insert_resource(Table::<SeekTime>);
        app.insert_resource(Table::<Delegated>);
        app.insert_resource(HitRegister);*/

        app.world.spawn_batch([
            (
                Sheet { start: p32(0.), duration:  p32(1000.), coverage: Coverage(0, 0) },
                Response { kind: ResponseKind::Commence, layer: 0 }
            ),
            (
                Sheet { start: p32(300.), duration:  p32(1000.), coverage: Coverage(1, 1) },
                Response { kind: ResponseKind::Commence, layer: 1 }
            ),
            (
                Sheet { start: p32(300.), duration:  p32(1000.), coverage: Coverage(2, 2) },
                Response { kind: ResponseKind::Switch, layer: 2 }
            ),
            (
                Sheet { start: p32(300.), duration:  p32(1000.), coverage: Coverage(3, 3) },
                Response { kind: ResponseKind::Toggle, layer: 3 }
            ),
            (
                Sheet { start: p32(300.), duration:  p32(1000.), coverage: Coverage(4, 4) },
                Response { kind: ResponseKind::Follow(p32(200.)), layer: 4 }
            ),
        ]);

        let hit = HitInfo {
            object_time: p32(200.),
            hit_time: p32(300.),
            layer: 3,
        };

        //app.insert_resource(SongTime(p32(500.)));

        app.update()
    }
}
