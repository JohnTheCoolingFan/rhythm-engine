use glam::{Vec2, Mat3}; 
use std::mem::swap;

pub enum Mode {
    Inactive,
    Hit(f32),
    Hold(f32, f32),
    Avoid(f32, f32),
}

pub struct CSplVertPairing {
    pub spline: usize,
    pub vertex: usize,
    pub scale: f32,
    pub rotation: f32
}

pub struct Properties {
    pub splines: Vec<CSplVertPairing>,
    pub rotation: usize,
    pub scale: usize,
    pub color: usize,
    pub grab: usize,
    pub mode: Mode,
}

pub struct PolyEntity {
    points: Vec<Vec2>,
    pub start: f32,
    pub duration: f32,
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
                points: initial.to_vec(),
                properties: Properties {
                    splines: vec![],
                    rotation: 0,
                    scale: 0,
                    color: 0,
                    grab: 0,
                    mode: Mode::Inactive,
                }
            })
        }
        else {
            None
        }
    }

    pub fn split_side(&self, mut n: usize, mut m: usize) {
        n += 2;
        m += 2;
        debug_assert!(
            n < self.points.len() && m < self.points.len(),
            "A chosen polygon vertex to split with does not exist"
        );

        if n < m { swap(&mut n, &mut m); }
        self.
    }
}
