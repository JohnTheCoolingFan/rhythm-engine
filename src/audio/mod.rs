use crate::GameState;
use bevy::prelude::*;
use bevy_kira_audio::prelude::{AudioPlugin as KiraAudio, *};
use noisy_float::prelude::R64;
use tap::Pipe;

#[derive(Resource, Default)]
struct SongChannel;

#[derive(Resource, Default, Deref, DerefMut)]
pub struct SongHandle(Handle<AudioInstance>);

#[derive(Default, Debug)]
pub struct ChartLoadEvent {
    pub chart_id: String,
    pub start_from: R64,
}

#[rustfmt::skip]
fn load_chart_song(
    state: Res<State<GameState>>,
    asset_server: Res<AssetServer>,
    song_channel: Res<AudioChannel<SongChannel>>,
    mut chart_load_events: EventReader<ChartLoadEvent>,
    mut handle: ResMut<SongHandle>,
) {
    if let Some(ChartLoadEvent { chart_id, start_from }) = chart_load_events.iter().last() {
        *handle = song_channel
            .play(asset_server.load(format!("charts/{}/song.ogg", chart_id)))
            .start_from(start_from.raw())
            .handle()
            .pipe(SongHandle);

        if matches!(state.0, GameState::Edit) {
            song_channel.pause();
        }
    }
}

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, game: &mut App) {
        game.add_plugin(KiraAudio)
            .init_resource::<SongHandle>()
            .add_audio_channel::<SongChannel>()
            .add_event::<ChartLoadEvent>()
            .add_system(load_chart_song);
    }
}
