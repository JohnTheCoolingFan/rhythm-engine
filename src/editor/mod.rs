mod playlist;
mod poly_entities;
mod tools;

use crate::{utils::*, GameState};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

#[derive(Default)]
enum Selection {
    #[default]
    Empty,
    Single(Entity),
    Multi(Vec<Entity>),
    SingleItem(Entity, usize),
    MultiItem(Entity, Vec<usize>),
}

#[derive(Default, Deref)]
struct ClipBoard(Selection);

#[derive(Default)]
struct ChannelScroll(u8);

#[derive(Default)]
struct ChannelSize(P32);

#[derive(Default, Resource)]
enum Focus {
    Polygons,
    #[default]
    Playlist,
    Channel(u8),
    Sheet(Entity),
}

struct Seeker {
    window_shift: T32,
}

struct Opacity {
    background: P32,
    background_participant: P32,
}

fn theme(mut contexts: EguiContexts) {
    catppuccin_egui::set_theme(contexts.ctx_mut(), catppuccin_egui::MACCHIATO);
}

fn tools(mut contexts: EguiContexts) {
    egui::SidePanel::left("tools")
        .resizable(false)
        .show(contexts.ctx_mut(), |ui| {
            ui.label("tools");
        });
}

fn playlist(mut contexts: EguiContexts) {
    egui::TopBottomPanel::bottom("playlist")
        .resizable(true)
        .show(contexts.ctx_mut(), |ui| {
            ui.label("playlist");
            ui.add(egui::Separator::default())
        });
}

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, game: &mut App) {
        game.init_resource::<Focus>()
            .add_systems((theme, playlist, tools).distributive_run_if(
                |state: Res<State<GameState>>| matches!(state.0, GameState::Edit),
            ));
    }
}
