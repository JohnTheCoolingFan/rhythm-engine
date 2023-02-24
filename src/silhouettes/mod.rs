use crate::{
    automation::{sequence::RGBA, Weight},
    harmonizer::*,
    hit::*,
    timing::*,
    utils::*,
};
use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology::TriangleList},
    sprite::MaterialMesh2dBundle,
};
use educe::*;
use noisy_float::{prelude::*, NoisyFloat};
use tap::{Conv, Pipe, Tap};

// This idea needs to cook more
// enum Tuning {
//     Translation {
//         mirror: bool,
//         rotation: R32,
//         magnification: R32,
//         ctrl: Option<usize>,
//     },
//     Rotation {
//         offset: R32,
//         rotation_ctrl: Option<usize>,
//         orientation_ctrl: Option<usize>,
//     },
//     Scale {
//         magnification: R32,
//         ctrl: Option<usize>,
//     },
// }

#[derive(Educe)]
#[educe(PartialEq, Ord, Eq, PartialOrd)]
#[derive(Clone)]
struct Group {
    label: String,
    #[educe(PartialEq(ignore), Ord(ignore), Eq(ignore), PartialOrd(ignore))]
    vertices: Ensured<Vec<usize>, FrontDupsDropped>,
}

struct Routing {
    channel: u8,
    target_group: usize,
    ctrl: Option<usize>,
    delimiter: bool,
}

#[derive(Component)]
struct PointCloud {
    points: Vec<Vec2>,
    groups: Ensured<Vec<Group>, FrontDupsDropped>,
    routings: Vec<Routing>,
}

struct DormantPoint {
    pos: Vec2,
    // Blending can get complicated with multiple routings.
    // Just Interpret the last 2 seen color and bloom values.
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
    RepeatingNgon {
        take: usize,
        step: usize,
    },
    Ngon {
        prompts: Vec<HitPrompt>,
        ctrl: usize,
    },
}

// Has to be its own component because this is what's responsible for play objects.
// Currrently no 2 play should overlap, storing in cloud would make this difficult to check.
// TODO:
//  - Per vertex coloring.
#[derive(Component)]
struct Activation {
    group: usize,
    z_offset: R32,
    base_color: Color,
    silhouette: Silhouette,
}

#[rustfmt::skip]
fn modulate(
    time_tables: ResMut<TimeTables>,
    modulations: Res<Table<Option<Modulation>>>,
    activations: Query<&TemporalOffsets, With<Activation>>,
    mut clouds: Query<(&PointCloud, &mut ModulationCache, &Children)>,
) {
    clouds
        .iter_mut()
        .filter(|(.., children)| children
            .iter()
            .flat_map(|entity| activations.get(*entity).ok())
            .any(|offsets| offsets.playable_at(time_tables.song_time))
        )
        .for_each(|(PointCloud { points, groups, routings }, mut cache, _)| {
            cache.clear();

            **cache = points
                .iter()
                .copied()
                .map(DormantPoint::new)
                .collect();

            routings.iter().for_each(|Routing { channel, target_group, ctrl, .. }| {
                let Some((indices, modulation)) = groups
                    .get(*target_group)
                    .map(|group| group.vertices.iter().copied())
                    .zip(modulations[*channel as usize].as_ref())
                else {
                    return
                };

                match modulation {
                    Modulation::RGBA(color) => indices.for_each(|index| {
                        cache[index].color = color
                            .map(NoisyFloat::raw)
                            .pipe(Some)
                    }),
                    Modulation::Luminosity(bloom) => indices.for_each(|index| {
                        cache[index].bloom = Some(*bloom)
                    }),
                    Modulation::Rotation(deg) => indices.clone().for_each(|index| {
                        cache[index].pos = ctrl
                            .map(|ctrl| cache[ctrl].pos)
                            .unwrap_or_else(|| indices.clone().map(|i| cache[i].pos).centroid())
                            .pipe(|pos| deg
                                .raw()
                                .to_radians()
                                .pipe(r32)
                                .pipe(|rad| cache[index].pos.rotate_about(pos, rad))
                            );
                    }),
                    Modulation::Scale(factor) => indices.clone().for_each(|index| {
                        cache[index].pos = ctrl
                            .map(|ctrl| cache[ctrl].pos)
                            .unwrap_or_else(|| indices.clone().map(|i| cache[i].pos).centroid())
                            .pipe(|pos| cache[index].pos.rotate_about(pos, *factor))
                    }),
                    Modulation::Translation(vec) => indices.for_each(|index| {
                        cache[index].pos += *vec
                    }),
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
    fn apply(&self, color: &mut [f32; 4], amount: T32) {
        color.iter_mut().take(3).for_each(|val| {
            *val += *val
                * self.vividness_curve.eval(amount).raw()
                * self.vividness_threshold.raw()
                + self.brightness_curve.eval(amount).raw()
                * self.brightness_threshold.raw()
        })
    }
}

#[rustfmt::skip]
fn render(
    time_tables: ResMut<TimeTables>,
    luminosity_settings: Res<LuminositySettings>,
    activations: Query<(&TemporalOffsets, &Activation, &Parent)>,
    clouds: Query<(&PointCloud, &ModulationCache)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    use Mesh::*;

    activations
        .iter()
        .filter(|(offsets, ..)| offsets.playable_at(time_tables.song_time))
        .flat_map(|(offsets, activation, parent)| clouds
            .get(parent.get())
            .map(|parent| (offsets, activation, parent))
        )
        .for_each(|(_, activation, (cloud, cache))| {
            let Some(vertices) = cloud
                .groups
                .get(activation.group)
                .map(|group| group
                    .vertices
                    .iter()
                    .map(|i| cache[*i].pos)
                    .map(|v| [v.x, v.y, 0.])
                    .collect::<Vec<_>>()
                )
            else {
                return
            };

            match &activation.silhouette {
                Silhouette::Ngon { prompts, ctrl } => {
                    commands.spawn(MaterialMesh2dBundle {
                        mesh: Mesh::new(TriangleList)
                            .tap_mut(|mesh| mesh.insert_attribute(ATTRIBUTE_POSITION, vertices)),
                        transform: Transform::default()
                            .with_translation(Vec3 { z: activation.z_offset, ..Default::default() })
                        ..default()
                    });
                },
                Silhouette::RepeatingNgon { take, step } => todo!(),
            }
        });

    let mut mesh = Mesh::new(TriangleList);
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![[-0.5, -0.5, 0.0], [-0.5, 0.5, 0.0], [0.5, 0.5, 0.0]],
    );

    let vertex_colors: Vec<[f32; 4]> = vec![
        Color::RED.as_rgba_f32(),
        Color::GREEN.as_rgba_f32(),
        Color::BLUE.as_rgba_f32(),
    ];

    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vertex_colors);
    mesh.set_indices(Some(Indices::U32(vec![0, 2, 1])));

    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(mesh).into(),
        transform: Transform::from_translation(Vec3::new(-96., 0., 0.))
            .with_scale(Vec3::splat(128.)),
        material: materials.add(ColorMaterial::default()),
        ..default()
    });
}
