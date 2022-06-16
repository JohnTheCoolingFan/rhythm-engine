use bevy::prelude::*;
use noisy_float::prelude::*;

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
