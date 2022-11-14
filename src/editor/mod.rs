use crate::map_selected;
use bevy::{ecs::schedule::ShouldRun, prelude::*};
use bevy_egui::{egui, EguiContext};
use bevy_system_graph::*;
use leafwing_input_manager::prelude::*;
use tap::{Conv, Tap};

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

#[derive(Default, Deref)]
struct Seeker(f64);

#[derive(Default, Deref, Resource)]
struct ChannelFocus(Option<u8>);

#[derive(Default)]
struct ChannelScroll(u8);

enum Action {
    Click,
}

fn sheet_arrangements(mut egui_context: ResMut<EguiContext>, focus: Res<ChannelFocus>) {
    if let None = **focus {
        egui::Window::new("Hello").show(egui_context.ctx_mut(), |ui| {
            ui.label("world");
        });
    }
}

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, game: &mut App) {
        game.init_resource::<ChannelFocus>().add_system_set(
            SystemGraph::new()
                .tap(|sysg| {
                    sysg.root(sheet_arrangements);
                })
                .conv::<SystemSet>()
                .with_run_criteria(map_selected),
        );
    }
}
