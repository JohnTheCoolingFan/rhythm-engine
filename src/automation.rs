use std::collections::BTreeSet;

use bevy::prelude::*;
use noisy_float::prelude::*;

use crate::hit::*;

pub trait Lerp {
    fn lerp(&self, other: Self, t: N32) -> Self;
}

enum Weight {
    Constant,
    Quadratic(N32),
    Cubic(N32),
}

struct Anchor {
    point: Vec3,
    weight: Weight,
}

struct Bound<T> {
    val: T,
    transition: Weight,
}

struct Automation<T> {
    start: N32,
    response: HitResponse,
    layer: u8,
    upper_bounds: BTreeSet<Bound<T>>,
    anchors: BTreeSet<Weight>,
}

#[derive(Component)]
pub struct Channel<T> {
    index: usize,
    clips: BTreeSet<Automation<T>>,
}

fn automation_system<T>(query: Query<&Channel<T>>)
where
    T: Lerp + Default + Send + Sync,
{
}

struct AutomationPlugin;

impl Plugin for AutomationPlugin {
    fn build(&self, app: &mut App) {}
}
