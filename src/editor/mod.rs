use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use leafwing_input_manager::prelude::*;

enum Selection {
    Single(Entity),
    Multi(Vec<Entity>),
    SingleItem(Entity, usize),
    MultiItem(Entity, Vec<usize>),
}

struct ClipBoard(Selection);

struct Seeker(f64);

struct ChannelFocus(u8);

struct ChannelScroll(u8);

enum Action {
    Click,
}

fn editor_root(mut egui_context: ResMut<EguiContext>) {
    egui::Window::new("Hello").show(egui_context.ctx_mut(), |ui| {
        ui.label("world");
    });
}

struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {}
}
