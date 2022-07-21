use crate::{sheet::*, utils::*, SongTime};

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
    /// Switches to a different automation permenantly with a start from the current time
    Switch(u8),
    /// Switches to a different automation but will switch back to the original
    /// automation on another hit. This can be repeated indefinetly
    Toggle(u8),
    /// Will stay at 0 state with no hit, for each hit it will play the automation
    /// from the hit time to hit time + excess.
    Follow(P32),
}

#[derive(Component)]
pub struct HitResponse {
    pub kind: ResponseKind,
    pub layer: u8,
}

#[derive(Component)]
pub enum ResponseState {
    Nil,
    Hit(P32),
    Delegated(bool),
}

#[derive(Clone, Copy)]
pub struct ResponseOutput {
    pub seek_time: P32,
    pub redirect: Option<u8>,
}

#[rustfmt::skip]
fn clear_hit_responses(
    time: Res<SongTime>,
    mut response_sheets: Query<(&SheetPosition, &mut ResponseState)>,
) {
    response_sheets
        .iter_mut()
        .filter(|(sheet, _)| !sheet.scheduled_at(**time))
        .for_each(|(_, mut response_state)| *response_state = ResponseState::Nil);
}

#[rustfmt::skip]
fn respond_to_hits(
    time: Res<SongTime>,
    hits: Res<HitRegister>,
    hit_resps: Query<&HitResponse>,
    mut sheets: Query<(
        &SheetPosition,
        &Instance<HitResponse>,
        &mut ResponseState
    )>,
)
    -> [ResponseOutput; 256]
{
    [ResponseOutput { seek_time: **time, redirect: None }; 256].tap_mut(|outputs| {
        sheets
            .iter_mut()
            .filter(|(sheet, _, _)| f32::EPSILON < sheet.duration.raw())
            .filter(|(sheet, _, _)| sheet.scheduled_at(**time))
            .map(|(sheet, instance, state)| (sheet, hit_resps.get(**instance).unwrap() ,state))
            .for_each(|(sheet, HitResponse { kind, layer }, mut state)| {
                use ResponseKind::*;
                use ResponseState::*;

                hits.iter()
                    .flatten()
                    .filter(|hit| sheet.scheduled_at(hit.hit_time) && hit.layer == *layer)
                    .for_each(|hit| match (kind, &mut *state) {
                        (Commence | Switch(_), state) => *state = Delegated(true),
                        (Toggle(_), Delegated(delegate)) => *delegate = !*delegate,
                        (Toggle(_), state) => *state = Delegated(true),
                        (Follow(_), last_hit) => *last_hit = Hit(hit.object_time),
                        _ => {}
                    });

                let adjusted_offset = match (kind, &mut *state) {
                    (Commence, Delegated(delegate)) if !*delegate => sheet.start,
                    (Follow(ex), Hit(hit)) if !(*hit..*hit + ex).contains(&**time) => *hit + ex,
                    _ => **time
                };

                let shift = match (kind, &mut *state) {
                    (Switch(shift) | Toggle(shift), Delegated(true)) => Some(*shift),
                    _ => None
                };

                sheet.coverage::<u8>().for_each(|index| outputs[index as usize] = ResponseOutput {
                    seek_time: adjusted_offset,
                    redirect: shift.map(|shift| index.wrapping_add(shift))
                })
            })
    })
}
