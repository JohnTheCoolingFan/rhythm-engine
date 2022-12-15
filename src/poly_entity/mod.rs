use crate::{harmonizer::*, utils::*};
use bevy::{math::DVec2, prelude::*};
use noisy_float::prelude::*;

enum GeomCtrl {
    Point(DVec2),
    Line { point: DVec2, angle: R64 },
}

enum VertexGroup {
    All,
    Select(Vec<usize>),
}

struct GroupRoutings {
    group: VertexGroup,
    geom_ctrl: GeomCtrl,
    routings: Ensured<Vec<u8>, Deduped>,
}

struct PolyEntity {
    vetices: Vec<DVec2>,
    routings: Vec<GroupRoutings>,
}

impl PolyEntity {
    fn modulate(&self, modulations: Table<Option<Modulation>>) -> Vec<DVec2> {
        // Propogate modulations to geom ctrl points in route queue
        todo!()
    }
}
