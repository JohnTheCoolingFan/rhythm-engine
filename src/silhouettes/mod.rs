use crate::{harmonizer::*, hit::*, timing::*, utils::*};
use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology::TriangleList},
    sprite::MaterialMesh2dBundle,
};
use educe::*;
use noisy_float::prelude::*;
use tap::Pipe;

// This idea needs to cook more
// enum Tuning {
//     Translation {
//         mirror: bool,
//         rotation: R64,
//         magnification: R64,
//         ctrl: Option<usize>,
//     },
//     Rotation {
//         offset: R64,
//         rotation_ctrl: Option<usize>,
//         orientation_ctrl: Option<usize>,
//     },
//     Scale {
//         magnification: R64,
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
    color: Option<Color>,
    bloom: Option<R32>,
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

#[derive(Component)]
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
    z_offset: R64,
    base_color: Color,
    offsets: TemporalOffsets,
    silhouette: Silhouette,
    cloud: Entity,
    geometries: Vec<Entity>,
}

#[rustfmt::skip]
fn modulate(
    time_tables: ResMut<TimeTables>,
    modulations: Res<Table<Option<Modulation>>>,
    activations: Query<&Activation>,
    mut commands: Commands,
    mut clouds: Query<(&PointCloud, &mut ModulationCache)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
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
