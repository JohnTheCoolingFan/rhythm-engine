#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use utils::*;

use bevy::{ecs::schedule::ShouldRun, prelude::*};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use derive_more::From;

mod editor;
mod hit;
mod sheet;
mod timing;
mod utils;

use sheet::SheetPlugin;

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
    Edit(String),
    Play(String),
    Paused,
}

fn map_selected(game_state: Res<State<GameState>>) -> ShouldRun {
    match game_state.current() {
        GameState::Edit(_) | GameState::Play(_) => ShouldRun::Yes,
        _ => ShouldRun::No,
    }
}

#[rustfmt::skip]
fn main() {
    let mut game = App::new();

    game.add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(SheetPlugin)
        .init_resource::<Settings>();

    #[cfg(debug_assertions)]
    game.add_state(GameState::Edit("test".to_string()));

    #[cfg(not(debug_assertions))]
    game.add_state(GameState::Browse);

    game.run()
}

fn ui_example(mut egui_context: ResMut<EguiContext>) {
    egui::Window::new("Hello").show(egui_context.ctx_mut(), |ui| {
        ui.label("world");
    });
}
/*fn setup_system(mut commands: Commands) {
    let shape = shapes::RegularPolygon {
        sides: 6,
        feature: shapes::RegularPolygonFeature::Radius(200.0),
        ..shapes::RegularPolygon::default()
    };

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(GeometryBuilder::build_as(
        &shape,
        DrawMode::Outlined {
            fill_mode: FillMode::color(Color::CYAN),
            outline_mode: StrokeMode::new(Color::BLACK, 10.0),
        },
        Transform::default(),
    ));
}*/
