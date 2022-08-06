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
    /// Stays at 0 state until hit, once hit which it will commece from the current time
    Commence,
    /// Switches to a delegate automation permenantly with a start from the current time
    Switch,
    /// Switches to a delegate automation but will switch back to the original
    /// automation on another hit. This can be repeated indefinetly
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

#[derive(Default, From, Deref, DerefMut, Clone, Copy)]
pub struct SeekTime(pub P32);

fn clear_hit_responses(mut instances: Query<(&Sheet, &mut ResponseState)>) {
    instances
        .iter_mut()
        .for_each(|(_, mut response_state)| *response_state = ResponseState::Nil);
}

#[rustfmt::skip]
fn respond_to_hits(
    song_time: Res<SongTime>,
    mut seek_times: ResMut<Table<SeekTime>>,
    mut delegations: ResMut<Table<Delegated>>,
    hits: Res<HitRegister>,
    responses: Query<&Response>,
    mut instances: Query<(
        &Sheet,
        &GenID<Response>,
        &mut ResponseState
    )>,
) {
    **seek_times = seek_times.map(|_| SeekTime(**song_time));
    **delegations = delegations.map(|_| Delegated(false));

    instances
        .iter_mut()
        .filter(|(sheet, ..)| f32::EPSILON < sheet.duration.raw())
        .filter(|(sheet, ..)| sheet.scheduled_at(**song_time))
        .map(|(sheet, gen_id, state)| (sheet, responses.get(**gen_id).unwrap() ,state))
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
                (Follow(ex), &Hit(hit)) if !(hit..hit + ex).contains(&**song_time) => hit + ex,
                _ => **song_time
            };

            let delegation = match (kind, &mut *state) {
                (Switch | Toggle, Delegated(state)) => *state,
                _ => false
            };

            sheet.coverage::<u8>().for_each(|index| {
                *(*seek_times)[index as usize] = adjusted_offset;
                *(*delegations)[index as usize] = delegation;
            })
        });
}
