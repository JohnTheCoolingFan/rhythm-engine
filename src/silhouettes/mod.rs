use core::iter::once as iter_once;

use crate::{
    audio::SongInfo,
    automation::{sequence::*, *},
    harmonizer::arranger::{ChannelCoverage, CoverageRange},
    harmonizer::*,
    hit::*,
    map_selected,
    timing::*,
    utils::*,
};

use bevy::{
    prelude::*,
    render::{
        mesh::{Indices::U16, MeshVertexAttribute},
        render_resource::PrimitiveTopology::TriangleList,
    },
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use lyon::{
    math::Point,
    path::{builder::*, Path},
    tessellation::*,
};

use educe::*;
use itertools::Itertools;
use noisy_float::{prelude::*, NoisyFloat};
use tap::{Conv, Pipe, Tap};

type VertexID = usize;
type GroupID = usize;
type TuningID = usize;

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
#[derive(Clone, Copy, Debug)]
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
        flip: bool,
    },
    #[educe(Ord(rank = 3))]
    Warp { target: GroupID },
    #[educe(Ord(rank = 4))]
    NA,
}

#[derive(Debug)]
struct Route {
    target_groups: Vec<(GroupID, Vec<TuningID>)>,
    tunings: Vec<Tuning>,
    channels: Vec<u8>,
}

#[derive(Component)]
pub struct PointCloud {
    points: Vec<Vec2>,
    groups: Vec<Group>,
    routes: Vec<Route>,
    children: Vec<Entity>,
}

#[derive(Clone, Copy, Default, Debug)]
struct InertPoint {
    pos: Vec2,
    color: Option<[f32; 4]>,
    lumin: Option<T32>,
}

impl InertPoint {
    fn new(pos: Vec2) -> Self {
        Self {
            pos,
            color: None,
            lumin: None,
        }
    }
}

#[derive(Deref, DerefMut, Component, Default, Debug)]
struct ModulationCache(Vec<InertPoint>);

enum Silhouette {
    Polygon,
    Curves {
        // TODO
    },
}

enum Property {
    NA,
    Prompt { prompts: Vec<HitPrompt> },
    Repeat { step: usize, take: usize },
}

#[derive(Component)]
pub struct Activation {
    z: R32,
    ctrl: VertexID,
    group: GroupID,
    base_color: [R32; 4],
    silhouette: Silhouette,
    property: Property,
    parent: Entity,
}

#[rustfmt::skip]
fn modulate(
    song_info: Res<SongInfo>,
    modulations: Res<Table<Option<Modulation>>>,
    activations: Query<&TemporalOffsets, With<Activation>>,
    mut clouds: Query<(&PointCloud, &mut ModulationCache)>,
) {
    let joined = clouds.iter_mut().filter(|(cloud, _)| cloud.children
        .iter()
        .flat_map(|entity| activations.get(*entity).ok())
        .any(|offsets| offsets.playable_at(song_info.pos))
    );

    joined.for_each(|(PointCloud { points, groups, routes, .. }, mut cache)| {
        cache.clear();

        **cache = points
            .iter()
            .copied()
            .map(InertPoint::new)
            .collect();

        let flattened = routes.iter().flat_map(|Route { channels, target_groups, tunings }| channels
            .iter()
            .cartesian_product(target_groups.iter())
            .map(|(channel, (group, tuning_indices))| (tuning_indices, (channel, group)))
            .flat_map(move |(indices, pairs)| indices
                .iter()
                .map(move |i| (tunings[*i], pairs))
                .chain(iter_once((Tuning::NA, pairs)).filter(|_| indices.is_empty()))
            )
        );

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
                    cache[index].lumin = Some(*bloom)
                }),
                Modulation::Rotation(theta) => {
                    let (ctrl, orient_ctrl) = match tuning {
                        Tuning::Rotation { ctrl, orient_ctrl } => (ctrl, orient_ctrl),
                        _ => (None, None)
                    };

                    let ctrl = ctrl.map(|ctrl| cache[ctrl].pos).unwrap_or_else(|| indices
                        .clone()
                        .map(|i| cache[i].pos)
                        .centroid()
                    );

                    let theta = r32(theta.raw().to_radians());

                    indices.clone().for_each(|i| {
                        cache[i].pos = cache[i].pos.rotate_about(ctrl, theta)
                    });

                    if let Some(orient_ctrl) = orient_ctrl.map(|i| cache[i].pos) {
                        indices.clone().for_each(|i| {
                            cache[i].pos = cache[i].pos.rotate_about(orient_ctrl, -theta)
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
                    let (angle, dilation, flip) = match tuning {
                        Tuning::Translation { angle, dilation, flip } => (angle, dilation, flip),
                        _ => (r32(0.), r32(1.), false),
                    };

                    let tuned_shift = shift
                        .rotate_about(Vec2::default(), r32(angle.raw().to_radians()))
                        .scale_about(Vec2::default(), dilation)
                        .tap_mut(|vec| if flip { vec.x = -vec.x });

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
    fn apply(&self, color: [f32; 4], amount: T32) -> [f32; 4] {
        color.tap_mut(|color| color.iter_mut().take(3).for_each(|val| {
            *val += *val
                * self.vividness_curve.eval(amount).raw()
                * self.vividness_threshold.raw()
                + self.brightness_curve.eval(amount).raw()
                * self.brightness_threshold.raw()
        }))
    }
}

#[rustfmt::skip]
fn render(
    song_info: Res<SongInfo>,
    luminosity_settings: Res<LuminositySettings>,
    activations: Query<(Entity, &TemporalOffsets, &Activation)>,
    clouds: Query<(&PointCloud, &ModulationCache)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {

    activations
        .iter()
        .filter(|(_, offsets, ..)| offsets.playable_at(song_info.pos))
        .flat_map(|(entity, offsets, activation)| clouds
            .get(activation.parent)
            .map(|parent| (entity, offsets, activation, parent))
        )
        .for_each(|(entity, _, activation, (cloud, cache))| {
            let Some(members) = cloud
                .groups
                .get(activation.group)
                .map(|group| &group.vertices)
            else {
                return
            };

            let compute_luminosity = |color: Option<[f32; 4]>, luminosity: Option<T32>| color
                .unwrap_or_else(|| activation.base_color.map(NoisyFloat::raw))
                .pipe(|color| luminosity_settings.apply(color, luminosity.unwrap_or(t32(0.))));

            let (take, step) = match activation.property {
                Property::Repeat { step, take } => (step, take),
                _ => (members.len(), members.len())
            };

            assert!(0 < step);
            assert!(3 <= take);

            match &activation.silhouette {
                Silhouette::Polygon => (0..)
                    .step_by(step)
                    .map_while(|start| members.get(start..start + take))
                    .for_each(|indices| {
                        let path = Path::builder_with_attributes(4).pipe(|mut builder| {
                            let start = cache[indices[0]];
                            builder.begin(
                                Point::new(start.pos.x, start.pos.y),
                                &compute_luminosity(start.color, start.lumin)
                            );

                            indices.iter().skip(1).map(|i| cache[*i]).for_each(|point| {
                                builder.line_to(
                                    Point::new(point.pos.x, point.pos.y),
                                    &compute_luminosity(point.color, point.lumin)
                                );
                            });

                            builder.line_to(
                                Point::new(start.pos.x, start.pos.y),
                                &compute_luminosity(start.color, start.lumin)
                            );

                            builder.close();
                            builder.build()
                        });

                        let mut colors = Vec::<[f32; 4]>::new();

                        let geometry = VertexBuffers::<[f32; 3], u16>::new().tap_mut(|geometry| {
                            Result::unwrap(FillTessellator::new().tessellate_path(
                                &path,
                                &FillOptions::default(),
                                &mut BuffersBuilder::new(geometry, ColorCtor::<0>::new(&mut colors))
                            ));
                        });

                        const POS: MeshVertexAttribute = Mesh::ATTRIBUTE_POSITION;
                        const COLOR: MeshVertexAttribute = Mesh::ATTRIBUTE_COLOR;

                        commands.entity(entity).insert(MaterialMesh2dBundle {
                            transform: Transform::default()
                                .with_translation(Vec3 { z: activation.z.raw(), ..default() }),
                            mesh: Mesh::new(TriangleList)
                                .tap_mut(|mesh| mesh.insert_attribute(POS, geometry.vertices))
                                .tap_mut(|mesh| mesh.insert_attribute(COLOR, colors))
                                .tap_mut(|mesh| mesh.set_indices(Some(U16(geometry.indices))))
                                .pipe(|mesh| meshes.add(mesh))
                                .conv::<Mesh2dHandle>(),
                            material: materials.add(ColorMaterial::default()),
                            ..default()
                        });
                    }),
                Silhouette::Curves { .. } => {
                    // TODO
                },
            }
        });
}

pub struct SilhouettePlugin;

#[rustfmt::skip]
impl Plugin for SilhouettePlugin {
    fn build(&self, game: &mut App) {
        game.init_resource::<LuminositySettings>()
            .add_systems(
                (modulate, render)
                .chain()
                .distributive_run_if(map_selected)
                .after(HarmonizerSet::PostArrange)
            );
    }
}

#[cfg(debug_assertions)]
pub mod debug {
    use super::*;
    use crate::harmonizer::arranger::Sources;

    #[rustfmt::skip]
    pub fn silhouettes_debug_setup(mut commands: Commands) {
        let [cloud, activation] = [(); 2].map(|_| commands.spawn_empty().id());

        commands.get_entity(cloud).unwrap().insert((
            ModulationCache::default(),
            PointCloud {
                children: vec![activation],
                points: [(0., 0.), (200., 0.), (0., 200.), (-200., 0.), (0., -200.)]
                    .iter()
                    .map(|(x, y)| Vec2::new(*x, *y))
                    .collect::<Vec<_>>(),
                groups: vec![
                    Group { label: String::from("left"), vertices: vec![3].into() },
                    Group { label: String::from("middle"), vertices: vec![2].into() },
                    Group { label: String::from("right"), vertices: vec![1].into() },
                    Group { label: String::from("quad"), vertices: vec![1, 2, 3, 4].into() },
                ],
                routes: vec![
                    Route { target_groups: vec![(0, vec![])], channels: vec![0], tunings: vec![], },
                    Route { target_groups: vec![(1, vec![])], channels: vec![1], tunings: vec![], },
                    Route { target_groups: vec![(2, vec![])], channels: vec![2], tunings: vec![], },
                    Route { target_groups: vec![(3, vec![])], channels: vec![3], tunings: vec![], },
                ],
            }
        ));

        commands.get_entity(activation).unwrap().insert((
            TemporalOffsets { start: p32(0.), duration: p32(1000.) },
            Activation {
                z: r32(0.),
                ctrl: 0,
                group: 3,
                base_color: [1., 1., 1., 1.].map(r32),
                silhouette: Silhouette::Polygon,
                property: Property::NA,
                parent: cloud,
            }
        ));

        let [red_source, red_instance] = [(); 2].map(|_| commands.spawn_empty().id());

        commands.get_entity(red_source).unwrap().insert((
            vec![Anchor { x: p32(0.), val: RGBA([1., 0., 0., 1.].map(t32)), ..default() }]
                .pipe(Automation)
                .pipe(Sequence),
        ));

        commands.get_entity(red_instance).unwrap().insert((
            TemporalOffsets { start: p32(0.), duration: p32(1000.) },
            ChannelCoverage(vec![CoverageRange::new(0, 0)].into()),
            PrimarySequence(Sources::<Sequence::<RGBA>> {
                main: GenID::new(red_source),
                delegation: None,
            })
        ));

        let [green_source, green_instance] = [(); 2].map(|_| commands.spawn_empty().id());

        commands.get_entity(green_source).unwrap().insert((
            vec![Anchor { x: p32(0.), val: RGBA([0., 1., 0., 1.].map(t32)), ..default() }]
                .pipe(Automation)
                .pipe(Sequence),
        ));

        commands.get_entity(green_instance).unwrap().insert((
            TemporalOffsets { start: p32(0.), duration: p32(1000.) },
            ChannelCoverage(vec![CoverageRange::new(1, 1)].into()),
            PrimarySequence(Sources::<Sequence::<RGBA>> {
                main: GenID::new(green_source),
                delegation: None,
            })
        ));

        let [blue_source, blue_instance] = [(); 2].map(|_| commands.spawn_empty().id());

        commands.get_entity(blue_source).unwrap().insert((
            vec![Anchor { x: p32(0.), val: RGBA([0., 0., 1., 1.].map(t32)), ..default() }]
                .pipe(Automation)
                .pipe(Sequence),
        ));

        commands.get_entity(blue_instance).unwrap().insert((
            TemporalOffsets { start: p32(0.), duration: p32(1000.) },
            ChannelCoverage(vec![CoverageRange::new(2, 2)].into()),
            PrimarySequence(Sources::<Sequence::<RGBA>> {
                main: GenID::new(blue_source),
                delegation: None,
            })
        ));

        let [lumin_source, lumin_instance] = [(); 2].map(|_| commands.spawn_empty().id());

        commands.get_entity(lumin_source).unwrap().insert((
            vec![Anchor { x: p32(0.), val: Luminosity::new(t32(1.0)), ..default() }]
                .pipe(Automation)
                .pipe(Sequence),
        ));

        commands.get_entity(lumin_instance).unwrap().insert((
            TemporalOffsets { start: p32(0.), duration: p32(1000.) },
            ChannelCoverage(vec![CoverageRange::new(3, 3)].into()),
            PrimarySequence(Sources::<Sequence::<Luminosity>> {
                main: GenID::new(lumin_source),
                delegation: None,
            })
        ));
    }
}
