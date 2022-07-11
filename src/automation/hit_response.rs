use bevy::prelude::*;
use noisy_float::prelude::*;

use crate::automation::*;
use crate::resources::*;

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
    Follow(R32),
}

#[derive(Component)]
pub struct HitResponse {
    pub kind: ResponseKind,
    pub layer: u8,
}

#[derive(Component)]
pub enum ResponseState {
    Nil,
    Hit(R32),
    Delegated(bool),
}
