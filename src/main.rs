#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::upper_case_acronyms)]
#![allow(dead_code)]

use bevy::{
    core_pipeline::{bloom::BloomSettings, clear_color::ClearColorConfig},
    ecs::schedule::ShouldRun,
    prelude::*,
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

use editor::*;
use harmonizer::HarmonizerPlugin;
use silhouettes::*;

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

fn debug_setup(commands: Commands) {
    silhouettes::debug::silhouettes_debug_setup(commands)
}

fn main() {
    let mut game = App::new();

    game.add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(HarmonizerPlugin)
        .add_plugin(EditorPlugin)
        .add_plugin(SilhouettePlugin)
        .init_resource::<Settings>()
        .add_startup_system(setup);

    #[cfg(debug_assertions)]
    game.add_startup_system(debug_setup)
        .add_state(GameState::Edit);

    #[cfg(not(debug_assertions))]
    game.add_state(GameState::Browse);

    game.run()
}
