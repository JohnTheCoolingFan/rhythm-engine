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

pub struct SongControl;

pub fn song_control(
    mut contexts: EguiContexts,
    mut song_info: ResMut<SongInfo>,
    mut instances: ResMut<Assets<KiraInstance>>,
    realestate: Res<Realestate<SongControl>>,
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

    egui::Window::new("Song")
        .collapsible(false)
        .constrain(true)
        .fixed_rect(egui::Rect::from(*realestate))
        .show(contexts.ctx_mut(), |ui| {
            ui.spacing_mut().slider_width = realestate.width().raw() * 0.9475;
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

#[rustfmt::skip]
fn reallocate_editor_realestate(
    window: Query<&Window>,
    mut song_control_realestate: ResMut<Realestate<SongControl>>,
) {
    let remaining = window
        .get_single()
        .unwrap()
        .pipe(|window| window.resolution.clone())
        .pipe(|res| [0., 0., res.width(), res.height()].map(p32))
        .pipe(|[x0, y0, x1, y1]| Realestate::<()>::new((x0, y0), (x1, y1)));

    let [_remaining, song_control] = remaining.horizontal_split([15., 1.].map(p32));
    *song_control_realestate = song_control.into();
}

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, game: &mut App) {
        game.init_resource::<Realestate<SongControl>>().add_systems(
            (theme, reallocate_editor_realestate, song_control).distributive_run_if(
                |state: Res<State<GameState>>| matches!(state.0, GameState::Edit),
            ),
        );
    }
}
