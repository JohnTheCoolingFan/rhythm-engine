mod clouds;
mod playlist;

use crate::{audio::*, utils::*, GameState};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use playlist::*;
use tap::{Pipe, Tap};

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
    song_info: Res<SongInfo>,
    realestate: Res<Realestate<SongControl>>,
    song_channel: Res<AudioChannel<SongChannel>>,
    mut instances: ResMut<Assets<KiraInstance>>,
    mut contexts: EguiContexts,
) {
    let slider_get_set = |new_pos| {
        if let Some((instance, new_pos)) = instances.get_mut(&song_info.handle).zip(new_pos) {
            instance.seek_to(new_pos);
        }
        song_info.pos.raw().into()
    };

    let slider_formater = |n: f64, _| {
        let hrs = n as i32 / (60 * 60);
        let mins = (n as i32 / 60) % 60;
        let secs = n as i32 % 60;
        let millis = (n.fract() * 1000.) as i32;
        format!("{hrs:02}:{mins:02}:{secs:02}:{millis:03}")
    };

    egui::Window::new(song_info.title.as_str())
        .collapsible(false)
        .title_bar(false)
        .fixed_rect(egui::Rect::from(*realestate))
        .show(contexts.ctx_mut(), |ui| {
            fixed_layout_bug_workaround(ui);

            egui::Visuals::default()
                .tap_mut(|visuals| visuals.window_rounding = egui::Rounding::none())
                .pipe(|visuals| ui.ctx().set_visuals(visuals));

            ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                if egui::RichText::new("\u{23F5}")
                    .size(ui.available_height() * 0.5)
                    .strong()
                    .pipe(egui::Button::new)
                    .pipe(|button| ui.add(button))
                    .clicked()
                {
                    song_channel.resume();
                }

                if egui::RichText::new("\u{23F8}")
                    .size(ui.available_height() * 0.5)
                    .strong()
                    .pipe(egui::Button::new)
                    .pipe(|button| ui.add(button))
                    .clicked()
                {
                    song_channel.pause();
                }

                ui.spacing_mut().slider_width = ui.available_width() - 105.;

                egui::Slider::from_get_set(0.0..=song_info.dur.raw().into(), slider_get_set)
                    .custom_formatter(slider_formater)
                    .pipe(|slider| ui.add(slider))
            })
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
        .pipe(|Window { resolution: res, .. }| (res.width(), res.height()))
        .pipe(|(width, height)| [0., 0., width, height].map(p32))
        .pipe(|[x0, y0, x1, y1]| Realestate::<()>::new((x0, y0), (x1, y1)));

    let [_remaining, song_control] = remaining.horizontal_split([23., 1.].map(p32));
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
