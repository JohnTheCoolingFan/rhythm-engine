use crate::{harmonizer::*, hit::*, timing::*, utils::*};
use bevy::{
    math::{DMat3, DVec2},
    prelude::*,
};
use noisy_float::prelude::*;
use std::cmp::PartialEq;
use tap::Tap;

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
struct PointCloud {
    points: Vec<DVec2>,
    groups: Ensured<Vec<Group>, Deduped>,
}

enum Silhouette {
    Polygon,
}

struct Activation {
    group: usize,
    z_offset: R64,
    base_color: Color,
    offsets: TemporalOffsets,
    prompts: Vec<HitPrompt>,
    silhouette: Silhouette,
}

enum ChannelListener {
    RGBA,
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
    target_group: usize,
    listener: ChannelListener,
}

#[derive(Component)]
struct ActivationSet {
    transform: DMat3,
    source: GenID<PointCloud>,
    routings: Vec<Routing>,
    vertex_cache: Vec<DVec2>,
    activations: Vec<Activation>,
}

impl ActivationSet {
    fn playable_at(&self, time: P64) -> bool {
        self.activations
            .iter()
            .any(|Activation { offsets, .. }| offsets.playable_at(time))
    }

    fn reset_cache(&mut self, cloud: &PointCloud) {
        self.vertex_cache.clear();
        self.vertex_cache.extend_from_slice(&cloud.points);
    }
}

#[rustfmt::skip]
fn modulate(
    time_tables: ResMut<TimeTables>,
    modulations: Res<Table<Option<Modulation>>>,
    clouds: Query<&PointCloud>,
    mut activations: Query<&mut ActivationSet>,
) {
    activations
        .iter_mut()
        .filter(|activation_set| activation_set.playable_at(time_tables.song_time))
        .for_each(|mut activation_set| {
            let cloud = clouds
                .get(*activation_set.source)
                .expect("Activation set source should not be stale");

            activation_set
                .tap_mut(|activation_set| activation_set.reset_cache(cloud))
                .routings
                .iter()
                .for_each(|Routing { channel, target_group, listener }| {
                    todo!()
                });
        });
}
