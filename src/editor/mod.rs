mod clouds;
mod playlist;

use crate::{audio::*, utils::*, GameState};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use playlist::*;
use tap::Pipe;

#[derive(Default, Clone, Copy, Deref, DerefMut, Resource)]
struct Selection(Option<Entity>);

#[derive(Default, Clone, Copy, Deref, DerefMut, Resource)]
struct ClipBoard(Selection);

#[derive(Default)]
struct ChannelSize(P32);

fn theme(mut contexts: EguiContexts) {
    catppuccin_egui::set_theme(contexts.ctx_mut(), catppuccin_egui::MACCHIATO);
}

pub fn song_control(
    mut contexts: EguiContexts,
    mut song_info: ResMut<SongInfo>,
    mut instances: ResMut<Assets<KiraInstance>>,
) {
    let slider_formater = |n: f64, _| {
        let (hrs, mins, secs, mills) = (
            n as i32 / (60 * 60),
            (n as i32 / 60) % 60,
            n as i32 % 60,
            (n.fract() * 1000.) as i32,
        );

        format!("{hrs:02}:{mins:02}:{secs:02}:{mills:03}")
    };

    egui::TopBottomPanel::bottom("Song").show(contexts.ctx_mut(), |ui| {
        ui.spacing_mut().slider_width = 1815.0;
        ui.vertical_centered(|ui| ui.label(song_info.title.as_str()));
        let mut pos = song_info.pos.raw();

        if egui::Slider::new(&mut pos, 0.0..=song_info.dur.raw())
            .custom_formatter(slider_formater)
            .pipe(|slider| ui.add(slider))
            .changed()
        {
            song_info.pos = p32(pos);
            if let Some(instance) = instances.get_mut(&song_info.handle) {
                instance.seek_to(pos.into());
            }
        }
    });
}

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, game: &mut App) {
        game.add_systems((theme, song_control).distributive_run_if(
            |state: Res<State<GameState>>| matches!(state.0, GameState::Edit),
        ));
    }
}
