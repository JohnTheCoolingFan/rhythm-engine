#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use harmonizer::SheetPlugin;
use utils::*;

use bevy::{ecs::schedule::ShouldRun, prelude::*};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use derive_more::From;

mod automation;
mod editor;
mod harmonizer;
mod hit;
mod serialization;
mod timing;
mod utils;

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

fn main() {
    let mut game = App::new();

    game.add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(SheetPlugin)
        .init_resource::<Settings>();

    #[cfg(debug_assertions)]
    game.add_state(GameState::Edit);

    #[cfg(not(debug_assertions))]
    game.add_state(GameState::Browse);

    game.run()
}

fn ui_example(mut egui_context: ResMut<EguiContext>) {
    egui::Window::new("Hello").show(egui_context.ctx_mut(), |ui| {
        ui.label("world");
    });
}
