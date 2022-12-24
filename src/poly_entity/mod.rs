use crate::{harmonizer::*, hit::*, timing::*, utils::*};
use bevy::{math::DVec2, prelude::*};
use noisy_float::prelude::*;
use tinyvec::TinyVec;

#[derive(Component)]
struct StaticPointCloud {
    points: Vec<DVec2>,
    groups: Vec<TinyVec<[usize; 6]>>,
}

#[derive(Default)]
struct Route {
    channel: u8,
    group: usize,
    local_ctrl: usize,
    offest_angle: R32,
}

#[derive(Component)]
struct Routing {
    target: GenID<StaticPointCloud>,
    routes: TinyVec<[Route; 1]>,
}

struct Activation {
    paths: TinyVec<[usize; 6]>,
    interval: TemporalInterval,
    hit_prompt: Vec<HitPrompt>,
}

#[derive(Component)]
struct ModulatedPointCloud {
    vertices: Vec<DVec2>,
    interval: TemporalInterval,
    activations: Vec<Activation>,
}

#[rustfmt::skip]
fn modulate(
    time_tables: ResMut<TimeTables>,
    modulations: Res<Table<Option<Modulation>>>,
    static_clouds: Query<&StaticPointCloud>,
    routing: Query<&Routing>,
    modulated_clouds: Query<&ModulatedPointCloud>,
) {
    todo!()
}
