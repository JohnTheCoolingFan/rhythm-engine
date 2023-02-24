#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::upper_case_acronyms)]
#![allow(dead_code)]

use bevy::{
    core_pipeline::{bloom::BloomSettings, clear_color::ClearColorConfig},
    ecs::schedule::ShouldRun,
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology::TriangleList},
    sprite::MaterialMesh2dBundle,
};
use bevy_egui::EguiPlugin;

mod automation;
mod editor;
mod harmonizer;
mod hit;
mod serialization;
mod silhouettes;
mod timing;
mod utils;

use automation::Weight;
use editor::*;
use harmonizer::HarmonizerPlugin;
use noisy_float::prelude::*;
use tap::Pipe;
use utils::*;

#[derive(Resource)]
struct Settings {
    ui_scale: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self { ui_scale: 1. }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum GameState {
    Browse,
    Edit,
    Play,
    Paused,
}

fn map_selected(game_state: Res<State<GameState>>) -> ShouldRun {
    match game_state.current() {
        GameState::Edit | GameState::Play => ShouldRun::Yes,
        _ => ShouldRun::No,
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        BloomSettings {
            intensity: 0.45,
            knee: 0.1,
            ..BloomSettings::default()
        },
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::Custom(Color::rgb(0.0, 0.0, 0.0)),
            },
            ..default()
        },
    ));
}

fn debug_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mul_weight = Weight::Quadratic(r32(-0.2));
    let add_weight = Weight::Quadratic(r32(5.0));

    let apply_bloom = |amount: f32, val: f32| -> f32 {
        val + val * mul_weight.eval(t32(amount)).raw() * 3.0
            + add_weight.eval(t32(amount)).raw() * 1.0
    };

    let add_alpha = |arr: [f32; 3]| -> [f32; 4] { [arr[0], arr[1], arr[2], 1.0] };

    for t in 0..=10 {
        let mut mesh = Mesh::new(TriangleList);
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![[-0.5, -0.5, 1.0], [0.0, 0.5, 1.0], [0.5, -0.5, 1.0]],
        );

        let vertex_colors: Vec<[f32; 4]> = vec![
            [1.0, 0.0, 0.0]
                .map(|v| apply_bloom(t as f32 / 10.0, v))
                .pipe(add_alpha),
            [0.0, 1.0, 0.0]
                .map(|v| apply_bloom(t as f32 / 10.0, v))
                .pipe(add_alpha),
            [0.0, 0.0, 1.0]
                .map(|v| apply_bloom(t as f32 / 10.0, v))
                .pipe(add_alpha),
        ];

        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vertex_colors);
        mesh.set_indices(Some(Indices::U32(vec![0, 2, 1])));

        commands.spawn(MaterialMesh2dBundle {
            mesh: meshes.add(mesh).into(),
            transform: Transform::default()
                .with_scale(Vec3::splat(100.))
                .with_translation(Vec3 {
                    x: -450. + 100. * t as f32,
                    z: 1.0,
                    ..Default::default()
                }),
            material: materials.add(ColorMaterial::default()),
            ..default()
        });
    }
}

fn main() {
    let mut game = App::new();

    game.add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(HarmonizerPlugin)
        .add_plugin(EditorPlugin)
        .init_resource::<Settings>()
        .add_startup_system(setup);

    #[cfg(debug_assertions)]
    game.add_startup_system(debug_setup)
        .add_state(GameState::Edit);

    #[cfg(not(debug_assertions))]
    game.add_state(GameState::Browse);

    game.run()
}
