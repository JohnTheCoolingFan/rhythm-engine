mod clouds;
mod playlist;

use crate::{utils::*, GameState};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use playlist::*;

#[derive(Default, Clone, Copy, Deref, DerefMut, Resource)]
struct Selection(Option<Entity>);

#[derive(Default, Clone, Copy, Deref, DerefMut, Resource)]
struct ClipBoard(Selection);

#[derive(Default)]
struct ChannelSize(P32);

fn theme(mut contexts: EguiContexts) {
    catppuccin_egui::set_theme(contexts.ctx_mut(), catppuccin_egui::MACCHIATO);
}

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, game: &mut App) {
        game.add_systems((theme, playlist_panel).distributive_run_if(
            |state: Res<State<GameState>>| matches!(state.0, GameState::Edit),
        ));
    }
}
