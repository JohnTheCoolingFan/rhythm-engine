use crate::utils::*;
use bevy::prelude::*;
use derive_more::From;
use noisy_float::prelude::*;

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
