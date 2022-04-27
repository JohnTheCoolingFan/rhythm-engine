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

pub enum HitResponse {
    Ignore,
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
    /// Will stay at 0 state with no hit, once hit it will play the automation
    /// from the hit time to hit time + excess.
    Follow {
        excess: N32,
        last_hit: Option<N32>,
    },
}

pub struct HitInfo {
    time: N32,
    layer: u8,
}

struct HitQueue(Vec<HitPrompt>);
