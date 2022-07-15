mod automation;
mod bound_sequence;
mod repeater;
mod spline;

use crate::{hit::*, utils::*};
use automation::*;
use bevy::prelude::*;
use bound_sequence::*;
use noisy_float::prelude::*;
use repeater::*;
use spline::*;

#[derive(Clone, Copy)]
struct Coverage(pub u8, pub u8);

struct Sheet {
    coverage: Coverage,
    start: P32,
    duration: P32,
    entity: Entity,
}

struct Arrangement {
    coverage: Coverage,
    hit_response: Option<Entity>,
    repeater: Option<Entity>,
    spline: Option<Entity>,
    anchors: Option<Entity>,
    color: Option<Entity>,
    luminosity: Option<Entity>,
    scale: Option<Entity>,
    rotation: Option<Entity>,
    geometry_ctrl: Option<Entity>,
}

struct Ensemble<'a> {
    coverage: Coverage,
    /// Alawys valid
    hit_response: Option<&'a HitResponse>,
    repeater: Option<&'a Repeater>,
    /// Exclusive
    spline: Option<&'a Spline>,
    automation: Option<&'a Automation<T32>>,
    /// Exclusive
    /// REQ: Some(_) = anchors
    color: Option<&'a BoundSequence<SpannedBound<Rgba>>>,
    luminosity: Option<&'a BoundSequence<SpannedBound<Luminosity>>>,
    scale: Option<&'a BoundSequence<ScalarBound<Scale>>>,
    rotation: Option<&'a BoundSequence<ScalarBound<Rotation>>>,
    /// Optional
    /// REQ: Some(_) = anchors && Some(_) = (rotation | scale)
    geometry_ctrl: Option<&'a GeometryCtrl>,
}

enum Modulation {
    Position(Vec2),
    Color(Rgba),
    Luminosity(Luminosity),
    Scale {
        magnitude: R32,
        ctrl: Option<Vec2>,
    },
    Rotation {
        theta: R32,
        ctrl: Option<Vec2>,
    },
}
