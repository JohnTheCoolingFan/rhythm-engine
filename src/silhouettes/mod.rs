use crate::{harmonizer::*, hit::*, timing::*, utils::*};
use bevy::{
    math::{DMat3, DVec2},
    prelude::*,
};
use educe::*;
use noisy_float::prelude::*;
use tap::Pipe;

#[derive(Educe)]
#[educe(PartialEq, Ord, Eq, PartialOrd)]
#[derive(Clone)]
struct Group {
    label: String,
    #[educe(PartialEq(ignore), Ord(ignore), Eq(ignore), PartialOrd(ignore))]
    vertices: Ensured<Vec<usize>, FrontDupsDropped>,
}

#[derive(Component)]
struct StencilCloud {
    points: Vec<DVec2>,
    groups: Ensured<Vec<Group>, FrontDupsDropped>,
}

#[derive(Clone, Copy)]
struct Regulations {
    translation_magnification: R64,
    translation_rotation: R64,
    scale_magnification: R64,
    rotation_offset: R64,
}

impl Default for Regulations {
    fn default() -> Self {
        Self {
            translation_magnification: r64(1.),
            translation_rotation: r64(0.),
            scale_magnification: r64(1.),
            rotation_offset: r64(0.),
        }
    }
}

#[derive(Educe)]
#[educe(PartialEq, Ord, Eq, PartialOrd)]
#[derive(Clone)]
struct Regulator {
    target_group: usize,
    #[educe(PartialEq(ignore), Ord(ignore), Eq(ignore), PartialOrd(ignore))]
    regulations: Regulations,
}

struct Routing {
    channel: u8,
    target_group: usize,
    ctrl: Option<usize>,
}

#[derive(Component)]
struct DormantCloud {
    parent: GenID<StencilCloud>,
    routings: Vec<Routing>,
    regulators: Ensured<Vec<Regulator>, FrontDupsDropped>,
    point_cache: Vec<DVec2>,
    transform: DMat3,
    children: Ensured<Vec<GenID<Activation>>, FrontDupsDropped>,
}

enum Silhouette {
    MultiNgon(usize),
    Ngon {
        prompts: Vec<HitPrompt>,
        ctrl: usize,
    },
}

#[derive(Component)]
struct Activation {
    parent: GenID<DormantCloud>,
    group: usize,
    z_offset: R64,
    base_color: Color,
    offsets: TemporalOffsets,
    silhouette: Silhouette,
}

impl DormantCloud {
    fn reset_cache(&mut self, cloud: &StencilCloud) {
        self.point_cache.clear();
        self.point_cache.extend_from_slice(&cloud.points);
    }
}

#[rustfmt::skip]
fn modulate(
    time_tables: ResMut<TimeTables>,
    modulations: Res<Table<Option<Modulation>>>,
    stencil_clouds: Query<&StencilCloud>,
    mut dormant_clouds: Query<&mut DormantCloud>,
    activations: Query<&Activation>,
) {
    let is_active = |dormant_cloud: &DormantCloud| dormant_cloud
        .children
        .iter()
        .any(|child| activations
            .get(**child)
            .expect("Hierarchy should be valid")
            .offsets
            .playable_at(time_tables.song_time)
        );
}
