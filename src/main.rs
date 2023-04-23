#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::upper_case_acronyms)]
#![allow(dead_code)]

use bevy::{
    asset::FileAssetIo,
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomPrefilterSettings, BloomSettings},
        clear_color::ClearColorConfig,
    },
    prelude::*,
    window::PresentMode,
};
use noisy_float::prelude::*;

use bevy_egui::EguiPlugin;
use bevy_screen_diagnostics::{ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin};

mod audio;
mod automation;
mod editor;
mod harmonizer;
mod hit;
mod serialization;
mod silhouettes;
mod timing;
mod utils;

use audio::*;
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

#[derive(Debug, Clone, Eq, PartialEq, Default, Hash, States)]
enum GameState {
    #[cfg(not(debug_assertions))]
    #[default]
    Browse,
    #[cfg(debug_assertions)]
    #[default]
    Edit,
    Play,
    Paused,
}

fn map_selected(game_state: Res<State<GameState>>) -> bool {
    matches!(game_state.0, GameState::Edit | GameState::Play)
}

fn setup(mut commands: Commands) {
    // TODO: Create dir structure
    let path = FileAssetIo::get_base_path();

    commands.spawn((
        BloomSettings {
            intensity: 0.3,
            composite_mode: BloomCompositeMode::Additive,
            prefilter_settings: BloomPrefilterSettings {
                threshold: 1.,
                threshold_softness: 0.1,
            },
            ..default()
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

fn debug_setup(mut commands: Commands) {
    commands.add(|world: &mut World| {
        world.send_event(ChartLoadEvent {
            chart_id: "000".into(),
            start_from: r64(0.),
        })
    });

    silhouettes::debug::silhouettes_debug_setup(commands);
}

fn main() {
    let mut game = App::new();

    game.add_state::<GameState>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rhythm Engine".into(),
                resolution: (1920., 1080.).into(),
                present_mode: PresentMode::AutoVsync,
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugin(AudioPlugin)
        .add_plugin(EguiPlugin)
        .add_plugin(HarmonizerPlugin)
        .add_plugin(SilhouettePlugin)
        .add_plugin(EditorPlugin)
        .init_resource::<Settings>()
        .add_startup_system(setup);

    #[cfg(debug_assertions)]
    game.add_plugin(ScreenDiagnosticsPlugin::default())
        .add_plugin(ScreenFrameDiagnosticsPlugin)
        .add_startup_system(debug_setup);

    game.run()
}
