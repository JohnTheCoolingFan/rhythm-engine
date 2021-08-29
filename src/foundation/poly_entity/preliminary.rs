use glam::Vec2;
use crate::foundation::Automation;

trait PolygonExtensions {
    fn clockwise(&self) -> bool;

}


impl PolygonExtensions for &[Vec2] {
    fn clockwise(&self) -> bool {
        0.0_f32 < self.iter()
            .skip(1)
            .enumerate()
            .map(|(i, p)| {
                let prev = self[i - 1];
                (p.x - prev.x) * (p.y + prev.y)
            })
            .sum()
    }

    
}

pub struct HitKeys {
    alphas: u8,
    phat: bool,
}

pub enum Beat {
    //0. <= pre <= 1.
    //pre + attack = activation time
    //pre + post = release time
    //no keys == lazy hit
    Hit {
        pre: f32,
        attack: f32,
        keys: Option<HitKeys>,
    },
    Hold {
        pre: f32,
        follow: Automation<f32>, //attack: start.x, post: end.x
        keys: Option<HitKeys>,
    },
    Avoid {
        pre: f32,
        attack: f32,
        post: f32,
    },
}

pub struct CSplVertPairing {
    pub spline: usize,
    pub vertex: usize,
    pub scale: f32,
    pub rotation: f32,
    pub x_invert: bool,
    pub y_invert: bool,
}

pub struct Properties {
    pub splines: Vec<CSplVertPairing>,
    pub rotation: Vec<Option<usize>>,
    pub scale: Vec<Option<usize>>,
    pub color: usize,
    pub glow: usize,
    pub beats: Vec<Beat>,
}

