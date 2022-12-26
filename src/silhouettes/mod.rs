use crate::{harmonizer::*, hit::*, timing::*, utils::*};
use bevy::{
    math::{DMat3, DVec2},
    prelude::*,
};
use noisy_float::prelude::*;
use std::cmp::PartialEq;
use tap::Pipe;

struct Group {
    label: String,
    vertices: Ensured<Vec<usize>, Deduped>,
}

impl PartialEq for Group {
    fn eq(&self, other: &Self) -> bool {
        self.label.eq(&other.label)
    }
}

#[derive(Component)]
struct StencilCloud {
    points: Vec<DVec2>,
    groups: Ensured<Vec<Group>, Deduped>,
}

enum Silhouette {
    Ngon(Vec<HitPrompt>),
    MultiNgon(u8),
}

enum Listener {
    Rgba,
    Luminosity,
    Translation {
        scale: Option<R64>,
        rotation: Option<R64>,
    },
    Scale {
        limit: Option<T64>,
        ctrl: Option<usize>,
    },
    Rotation {
        offset: Option<R64>,
        ctrl: Option<usize>,
    },
}

struct Routing {
    channel: u8,
    listener: Listener,
    target_group: usize,
}

#[derive(Component)]
struct DormantCloud {
    parent: GenID<StencilCloud>,
    routings: Vec<Routing>,
    point_cache: Vec<DVec2>,
    transform: DMat3,
    children: Ensured<Vec<GenID<Activation>>, Deduped>,
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

    dormant_clouds.iter_mut().filter(|cloud| is_active(cloud)).for_each(|mut dormant| {
        let stencil = stencil_clouds.get(*dormant.parent).expect("Hierarchy should be valid");

        dormant.reset_cache(stencil);

        let StencilCloud { groups, .. } = stencil;
        let DormantCloud { routings, point_cache, transform, .. } = &*dormant;

        routings.iter().for_each(|Routing { channel, listener, target_group }| {
            let Some((group, modulation)) = groups
                .get(*target_group)
                .zip(modulations[*channel as usize].as_ref())
            else {
                return
            };

            match (listener, modulation) {
                (Listener::Rgba, Modulation::Rgba(color)) => todo!("Make vertex cache with colors"),
                _ => todo!()
            }
        })
    })
}
