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

#[derive(Clone)]
pub struct HitInfo {
    /// Object time is used instead of hit time to keep animations synced with music
    pub object_time: R32,
    pub hit_time: R32,
    pub layer: u8,
}

pub enum HitReaction {
    /// Stays at 0 state until hit, once hit which it will commece from the current time
    Commence {
        started: bool,
    },
    /// Switches to a different automation permenantly with a start from the current time
    Switch {
        delegate: u8,
        switched: bool,
    },
    /// Switches to a different automation but will switch back to the original
    /// automation on another hit. This can be repeated indefinetly
    Toggle {
        delegate: u8,
        switched: bool,
    },
    /// Will stay at 0 state with no hit, for each hit it will play the automation
    /// from the hit time to hit time + excess.
    Follow {
        excess: R32,
        last_hit: Option<R32>,
    },
}

impl HitReaction {
    pub fn react(&mut self, HitInfo { object_time, .. }: &HitInfo) {
        match self {
            Self::Commence { started } => *started = true,
            Self::Switch { switched, .. } => *switched = true,
            Self::Toggle { switched, .. } => *switched = !*switched,
            Self::Follow { last_hit, .. } => *last_hit = Some(*object_time),
        }
    }

    pub fn delegate(&self) -> Option<u8> {
        match self {
            Self::Switch { delegate, switched } | Self::Toggle { delegate, switched } => {
                switched.then(|| *delegate)
            }
            _ => None,
        }
    }

    #[rustfmt::skip]
    pub fn translate(&self, offset: R32) -> R32 {
        match self {
            Self::Commence { started } => if *started { offset } else { r32(0.) },
            Self::Follow { excess, last_hit } => last_hit.map_or(offset, |last_hit| {
                if (last_hit..last_hit + excess).contains(&offset) {
                    offset
                } else {
                    last_hit + excess
                }
            }),
            _ => offset
        }
    }
}
