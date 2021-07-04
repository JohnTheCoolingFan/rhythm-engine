use glam::{Vec2, Mat3}; 
use std::mem::swap;

pub enum Mode {
    Hit(f32, f32),
    Hold(f32, f32, f32),
    Avoid(f32, f32, f32),
}

pub enum BeatKeys {
    K1,
    K2,
    K3,
    Phat
}

pub struct Beat {
    mode: Mode,
    keys: BeatKeys
}

pub struct CSplVertPairing {
    pub spline: usize,
    pub vertex: usize,
    pub scale: f32,
    pub rotation: f32
}

pub struct Properties {
    pub splines: Option<Vec<CSplVertPairing>>,
    pub rotation: Option<usize>,
    pub scale: Option<usize>,
    pub grab: Option<usize>,
    pub color: usize,
    pub beats: Vec<Beat>,
}

pub struct PolyEntity {
    points: Vec<Vec2>,
    pub start: f32,
    pub duration: f32,
    pub local_center: Vec2,
    pub properties: Properties
}

impl PolyEntity {
    #[rustfmt::skip]
    pub fn new(start: f32, duration: f32, initial: &[Vec2; 3]) -> Option<Self> {
        if Mat3::from_cols_array(&[
            initial[0].x, initial[0].y, 1.,
            initial[1].x, initial[1].y, 1.,
            initial[2].x, initial[2].y, 1.,
        ]).transpose().determinant() > 0. {
            Some(Self {
                start,
                duration,
                local_center: initial.into_iter().sum::<Vec2>() / initial.len() as f32,
                points: initial.to_vec(),
                properties: Properties {
                    splines: None,
                    rotation: None,
                    scale: None,
                    color: 0,
                    grab: None,
                    beats: vec![]
                }
            })
        }
        else {
            None
        }
    }

    pub fn try_split_side(&mut self, mut n: usize, mut m: usize) -> Result<(), ()> {
        n += 1;
        m += 1;
        debug_assert!(
            n < self.points.len() && m < self.points.len(),
            "A chosen polygon vertex to split with does not exist"
        );
        if m < n { swap(&mut n, &mut m) }
        if m - n == 1 {
            self.points.insert(m, self.points[n].lerp(self.points[m], 0.5));
            Ok(())
        }
        else {
            Err(())
        }
    }

    pub fn set_vertex(&mut self, n: usize, pos: Vec2) {
        self.points[n + 1] = pos;
    }

    pub fn set_position(&mut self, pos: Vec2) {
        self.points[0] = pos;
    }
}
