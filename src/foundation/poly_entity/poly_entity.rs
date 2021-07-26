use crate::{foundation::Automation, utils::math::*};
use glam::{Mat3, Vec2};
use std::mem::swap;

struct HitKeys {
    alphas: u8,
    phat: bool,
}

pub enum Beat {
    //0. <= pre <= 1.
    //start + attack = activation time
    //start + post = release time
    //no keys == lazy hit
    Hit {
        pre: f32,
        attack: f32,
        keys: Option<HitKeys>,
    },
    Hold {
        pre: f32,
        follow: Automation, //attack: start.x, post: end.x
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
    pub rotation: Option<usize>,
    pub scale: Option<usize>,
    //pub grab: Option<usize>,
    //pub nudge: Option<usize>,
    pub color: usize,
    pub glow: usize,
    pub beats: Vec<Beat>,
}

pub struct PolyEntity {
    points: Vec<Vec2>, //contains position offset
    pub start: f32,
    pub duration: f32,
    pub local_center: Vec2,
    pub properties: Properties,
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
                    splines: vec![],
                    rotation: None,
                    scale: None,
                    color: 0,
                    glow: 0,
                    //grab: None,
                    //nudge: None,
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
        if m < n {
            swap(&mut n, &mut m)
        }
        if m - n == 1 {
            self.points
                .insert(m, self.points[n].lerp(self.points[m], 0.5));
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn set_vertex(&mut self, n: usize, pos: Vec2) {
        self.points[n + 1] = pos - self.points[0];
    }

    pub fn set_position(&mut self, pos: Vec2) {
        self.points[0] = pos;
    }

    pub fn rotate(&mut self, deg: f32) {
        for point in &mut self.points.iter_mut().skip(1) {
            *point = point.rotate_about(&self.local_center, deg);
        }
    }

    pub fn scale(&mut self, factor: f32) {
        for point in self.points.iter_mut().skip(1) {
            *point = point.scale_about(&self.local_center, factor)
        }
    }
}
