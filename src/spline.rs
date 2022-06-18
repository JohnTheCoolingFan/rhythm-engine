use bevy::prelude::*;
use noisy_float::prelude::*;

use crate::utils::*;

enum Curvature {
    Linear,
    Circular(Vec2),
    Quadratic(Vec2),
    Cubic(Vec2, Vec2),
}

struct Segment {
    curvature: Curvature,
    tollerence: R32,
    end: Vec2,
}

struct Spline(Vec<Segment>);

enum RenderedSegement {
    Points {
        path_offset: R32,
        points: Vec<Vec2>,
    },
    Arc {
        path_offset: R32,
        start: Vec2,
        theta: Option<R32>,
        end: Vec2,
    },
}

impl Quantify for RenderedSegement {
    fn quantify(&self) -> R32 {
        match self {
            Self::Points { path_offset, .. } | Self::Arc { path_offset, .. } => *path_offset,
        }
    }
}

struct SplineLut(Vec<RenderedSegement>);
