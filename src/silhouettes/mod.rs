use crate::{harmonizer::*, hit::*, timing::*, utils::*};
use bevy::{math::DVec2, prelude::*};
use noisy_float::prelude::*;
use std::collections::HashMap;
use tinyvec::TinyVec;

struct Label(String);

struct Groups(Vec<(Ensured<Vec<usize>, Deduped>, Label)>);

#[derive(Component)]
struct PointCloud {
    points: Vec<DVec2>,
    groups: Groups,
}

enum Silhouette {
    Polygon,
}

struct Activation {
    group: usize,
    offsets: TemporalOffsets,
    hit_prompt: Vec<HitPrompt>,
    silhouette: Silhouette,
}

struct Routing {
    target_group: usize,
    channel: u8,
    ctrl_vertex: usize,
    offest_angle: R32,
}

#[derive(Component)]
struct ActivationSet {
    source: GenID<PointCloud>,
    vertex_cache: Vec<DVec2>,
    activations: Vec<Activation>,
    routings: Vec<Routing>,
}

impl ActivationSet {
    pub fn playable_at(&self, time: P64) -> bool {
        self.activations
            .iter()
            .any(|Activation { offsets, .. }| offsets.playable_at(time))
    }
}

#[rustfmt::skip]
fn modulate(
    time_tables: ResMut<TimeTables>,
    modulations: Res<Table<Option<Modulation>>>,
    clouds: Query<&PointCloud>,
    activations: Query<&ActivationSet>,
) {
    todo!()
}
