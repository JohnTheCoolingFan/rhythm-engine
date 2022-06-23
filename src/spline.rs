use bevy::prelude::*;
use derive_more::{Deref, DerefMut, From};
use itertools::Itertools;
use noisy_float::prelude::*;

use crate::{resources::*, utils::*};

enum Curvature {
    Linear,
    Circular(Vec2),
    Quadratic(Vec2),
    Cubic(Vec2, Vec2),
}

struct Segment {
    curvature: Curvature,
    end: Vec2,
}

pub enum CurveRender {
    Points(Vec<(Vec2, R32)>),
    Arc {
        start: Vec2,
        center: Vec2,
        theta: Option<R32>,
    },
}

pub struct SegmentRender {
    path: CurveRender,
    start: R32,
}

impl SegmentRender {
    fn length(&self) -> R32 {
        match self.path {
            CurveRender::Points(data) => data.last().unwrap().1,
            CurveRender::Arc { .. } => unimplemented!(),
        }
    }
}

impl Quantify for SegmentRender {
    fn quantify(&self) -> R32 {
        self.start
    }
}

#[derive(Component, Deref, DerefMut, From)]
struct Spline(Vec<Segment>);

#[derive(Component, Deref, DerefMut, From)]
pub struct SplineLut(pub Vec<SegmentRender>);

impl Spline {
    #[rustfmt::skip]
    fn render(&self) -> SplineLut {
        [Segment { curvature: Curvature::Linear, end: Vec2::new(0., 0.) }]
            .iter()
            .chain(self.iter())
            .tuple_windows::<(_, _)>()
            .scan(r32(0.), |length, (prev, current)| {
                unimplemented!()
            })
            .collect::<Vec<_>>()
            .into()
    }
}

#[derive(Component)]
struct SplineIndexCache {
    segment: usize,
    path: Option<usize>,
}

fn sync_spline_luts(splines: Query<Changed<Spline>>, mut luts: ResMut<SplineLuts>) {
    unimplemented!()
}
