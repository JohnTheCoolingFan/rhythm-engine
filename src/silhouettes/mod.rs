use crate::{
    automation::{sequence::RGBA, Weight},
    harmonizer::*,
    hit::*,
    timing::*,
    utils::*,
};

use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, MeshVertexAttribute},
        render_resource::PrimitiveTopology::TriangleList,
    },
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use lyon::{
    math::{point, Point},
    path::{builder::*, Path},
    tessellation::*,
};

use educe::*;
use itertools::Itertools;
use noisy_float::{prelude::*, NoisyFloat};
use tap::{Conv, Pipe, Tap};

type VertexID = usize;
type GroupID = usize;

#[derive(Educe)]
#[educe(PartialEq, Ord, Eq, PartialOrd)]
#[derive(Clone)]
struct Group {
    label: String,
    #[educe(PartialEq(ignore), Ord(ignore), Eq(ignore), PartialOrd(ignore))]
    vertices: Ensured<Vec<VertexID>, StableDeduped>,
}

#[derive(Educe)]
#[educe(PartialEq, Ord, Eq, PartialOrd)]
#[derive(Clone, Copy)]
enum Tuning {
    #[educe(Ord(rank = 0))]
    Scale {
        ctrl: Option<VertexID>,
        dilation: R32,
    },
    #[educe(Ord(rank = 1))]
    Rotation {
        ctrl: Option<VertexID>,
        orient_ctrl: Option<VertexID>,
    },
    #[educe(Ord(rank = 2))]
    Translation {
        angle: R32,
        dilation: R32,
        twist: bool,
    },
    #[educe(Ord(rank = 3))]
    Warp { target: GroupID },
}

struct Route {
    target_groups: Vec<(GroupID, Vec<usize>)>,
    tunings: Vec<Tuning>,
    channels: Vec<u8>,
}

#[derive(Component)]
struct PointCloud {
    points: Vec<Vec2>,
    groups: Ensured<Vec<Group>, FrontDupsDropped>,
    routes: Vec<Route>,
}

struct DormantPoint {
    pos: Vec2,
    color: Option<[f32; 4]>,
    bloom: Option<T32>,
}

impl DormantPoint {
    fn new(pos: Vec2) -> Self {
        Self {
            pos,
            color: None,
            bloom: None,
        }
    }
}

#[derive(Deref, DerefMut, Component)]
struct ModulationCache(Vec<DormantPoint>);

enum Silhouette {
    Polygon,
    Curves {
        // TODO
    },
}

enum Property {
    NA,
    Prompt { prompts: Vec<HitPrompt> },
    Repeat { take: usize, step: usize },
}

#[derive(Component)]
struct Activation {
    ctrl: VertexID,
    group: GroupID,
    z_offset: R32,
    base_color: [R32; 4],
    silhouette: Silhouette,
    property: Property,
}

#[rustfmt::skip]
fn modulate(
    time_tables: ResMut<TimeTables>,
    modulations: Res<Table<Option<Modulation>>>,
    activations: Query<&TemporalOffsets, With<Activation>>,
    mut clouds: Query<(&PointCloud, &mut ModulationCache, &Children)>,
) {
    let joined = clouds.iter_mut().filter(|(.., children)| children
        .iter()
        .flat_map(|entity| activations.get(*entity).ok())
        .any(|offsets| offsets.playable_at(time_tables.song_time))
    );

    joined.for_each(|(PointCloud { points, groups, routes }, mut cache, _)| {
        cache.clear();

        **cache = points
            .iter()
            .copied()
            .map(DormantPoint::new)
            .collect();

        let flattened = routes.iter().flat_map(|Route { channels, target_groups, tunings }| {
            channels
                .iter()
                .cartesian_product(target_groups.iter())
                .map(|(channel, (group, tuning_indices))| (tuning_indices, (channel, group)))
                .flat_map(move |(indices, pairs)| indices.iter().map(move |i| (tunings[*i], pairs)))
        });

        flattened.for_each(|(tuning, (channel, group))| {
            // TODO: Warping

            let Some((indices, modulation)) = groups
                .get(*group)
                .map(|group| group.vertices.iter().copied())
                .zip(modulations[*channel as usize].as_ref())
            else {
                return
            };

            match modulation {
                Modulation::RGBA(color) => indices.for_each(|index| {
                    cache[index].color = Some(color.map(NoisyFloat::raw))
                }),
                Modulation::Luminosity(bloom) => indices.for_each(|index| {
                    cache[index].bloom = Some(*bloom)
                }),
                Modulation::Rotation(deg) => {
                    let (ctrl, orient_ctrl) = match tuning {
                        Tuning::Rotation { ctrl, orient_ctrl } => (ctrl, orient_ctrl),
                        _ => (None, None)
                    };

                    let ctrl = ctrl.map(|ctrl| cache[ctrl].pos).unwrap_or_else(|| indices
                        .clone()
                        .map(|i| cache[i].pos)
                        .centroid()
                    );

                    let rad = r32(deg.raw().to_radians());

                    indices.clone().for_each(|i| {
                        cache[i].pos = cache[i].pos.rotate_about(ctrl, rad)
                    });

                    if let Some(orient_ctrl) = orient_ctrl.map(|i| cache[i].pos) {
                        indices.clone().for_each(|i| {
                            cache[i].pos = cache[i].pos.rotate_about(orient_ctrl, -rad)
                        })
                    }
                },
                Modulation::Scale(factor) => {
                    let (ctrl, dilation) = match tuning {
                        Tuning::Scale { ctrl, dilation } => (ctrl, dilation),
                        _ => (None, *factor)
                    };

                    let ctrl = ctrl.map(|ctrl| cache[ctrl].pos).unwrap_or_else(|| indices
                        .clone()
                        .map(|i| cache[i].pos)
                        .centroid()
                    );

                    indices.clone().for_each(|index| {
                        cache[index].pos = cache[index].pos.scale_about(ctrl, *factor * dilation)
                    })
                },
                Modulation::Translation(shift) => {
                    let (angle, dilation, twist) = match tuning {
                        Tuning::Translation { angle, dilation, twist } => (angle, dilation, twist),
                        _ => (r32(0.), r32(1.), false),
                    };

                    let tuned_shift = shift
                        .rotate_about(Vec2::default(), r32(angle.raw().to_radians()))
                        .scale_about(Vec2::default(), dilation)
                        .tap_mut(|vec| if twist { vec.x = -vec.x });

                    indices.for_each(|index| cache[index].pos += tuned_shift)
                },
                Modulation::Invalid => {}
            }
        })
    });
}

#[derive(Resource)]
struct LuminositySettings {
    vividness_curve: Weight,
    vividness_threshold: R32,
    brightness_curve: Weight,
    brightness_threshold: R32,
}

impl Default for LuminositySettings {
    fn default() -> Self {
        Self {
            vividness_curve: Weight::Quadratic(r32(-0.2)),
            vividness_threshold: r32(3.0),
            brightness_curve: Weight::Quadratic(r32(5.0)),
            brightness_threshold: r32(1.0),
        }
    }
}

impl LuminositySettings {
    #[rustfmt::skip]
    fn apply_bloom(&self, color: [f32; 4], amount: T32) -> [f32; 4] {
        color.tap_mut(|color| color.iter_mut().take(3).for_each(|val| {
            *val += *val
                * self.vividness_curve.eval(amount).raw()
                * self.vividness_threshold.raw()
                + self.brightness_curve.eval(amount).raw()
                * self.brightness_threshold.raw()
        }))
    }
}

const ATTR_POS: MeshVertexAttribute = Mesh::ATTRIBUTE_POSITION;
const ATTR_COL: MeshVertexAttribute = Mesh::ATTRIBUTE_COLOR;

#[rustfmt::skip]
fn render(
    time_tables: ResMut<TimeTables>,
    luminosity_settings: Res<LuminositySettings>,
    activations: Query<(Entity, &TemporalOffsets, &Activation, &Parent)>,
    clouds: Query<(&PointCloud, &ModulationCache)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {

    activations
        .iter()
        .filter(|(_, offsets, ..)| offsets.playable_at(time_tables.song_time))
        .flat_map(|(entity, offsets, activation, parent)| clouds
            .get(parent.get())
            .map(|parent| (entity, offsets, activation, parent))
        )
        .for_each(|(entity, _, activation, (cloud, cache))| {
            let compute_bloom = |color: Option<[f32; 4]>, bloom: Option<T32>| color
                .unwrap_or_else(|| activation.base_color.map(NoisyFloat::raw))
                .pipe(|color| luminosity_settings.apply_bloom(color, bloom.unwrap_or(t32(0.))));

            let Some(vertices) = cloud
                .groups
                .get(activation.group)
                .map(|group| &group.vertices)
            else {
                return
            };

            let (take, step) = match activation.property {
                Property::Repeat { take, step } => (take, step),
                _ => (vertices.len(), vertices.len())
            };

            /*match &activation.silhouette {
                Silhouette::Polygon => {
                    // TODO: Indices
                    commands.entity(entity).insert(MaterialMesh2dBundle {
                        transform: Transform::default()
                            .with_translation(Vec3 { z: activation.z_offset.raw(), ..default() }),
                        mesh: Mesh::new(TriangleList)
                            .tap_mut(|mesh| mesh.insert_attribute(ATTR_POS, vertices))
                            .tap_mut(|mesh| mesh.insert_attribute(ATTR_COL, colors))
                            .pipe(|mesh| meshes.add(mesh))
                            .conv::<Mesh2dHandle>(),
                        material: materials.add(ColorMaterial::default()),
                        ..default()
                    });
                },
                Silhouette::RepeatingNgon { take, step } => {
                    // TODO
                },
            }*/
        });
}
