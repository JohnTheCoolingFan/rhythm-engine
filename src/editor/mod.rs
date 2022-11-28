use crate::{utils::*, GameState};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

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
struct ChannelSize(P64);

#[derive(Default, Resource)]
enum Focus {
    #[default]
    Polygons,
    Playlist {
        channel: Option<u8>,
    },
}

struct Seeker {
    screen_shift: T64,
}

struct Opacity {
    background: P64,
    background_participant: P64,
}

fn tools(mut egui_context: ResMut<EguiContext>) {
    egui::SidePanel::left("tools")
        .resizable(false)
        .show(egui_context.ctx_mut(), |ui| {
            ui.label("tools");
        });
}

fn sheets(mut egui_context: ResMut<EguiContext>, focus: Res<Focus>) {}

fn playlist(mut egui_context: ResMut<EguiContext>, focus: Res<Focus>) {
    egui::TopBottomPanel::bottom("playlist")
        .resizable(true)
        .show(egui_context.ctx_mut(), |ui| {
            ui.label("playlist");
        });
}

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, game: &mut App) {
        game.init_resource::<Focus>().add_system_set(
            SystemSet::on_update(GameState::Edit)
                .with_system(playlist)
                .with_system(tools),
        );
    }
}
