use crate::{utils::*, GameState};
use bevy::{asset::FileAssetIo, prelude::*};
pub use bevy_kira_audio::prelude::{
    AudioInstance as KiraInstance, AudioPlugin as KiraPlugin, AudioSource as KiraSource, *,
};
use noisy_float::prelude::R64;
use tap::{Pipe, Tap};

#[derive(Resource, Default)]
pub struct SongChannel;

#[derive(Resource, Default, Debug)]
pub struct SongInfo {
    pub pos: P32,
    pub dur: P32,
    pub title: String,
    pub handle: Handle<KiraInstance>,
}

#[derive(Default, Debug)]
pub struct ChartLoadEvent {
    pub chart_id: String,
    pub start_from: R64,
}

fn load_chart_song(
    state: Res<State<GameState>>,
    song_channel: Res<AudioChannel<SongChannel>>,
    mut chart_load_events: EventReader<ChartLoadEvent>,
    mut kira_sources: ResMut<Assets<KiraSource>>,
    mut song_info: ResMut<SongInfo>,
) {
    let Some(ChartLoadEvent { chart_id, start_from }) = chart_load_events
        .iter()
        .last()
    else {
        return
    };

    let Ok(source) = FileAssetIo::get_base_path()
        .tap_mut(|path| path.push("assets"))
        .tap_mut(|path| path.push("charts"))
        .tap_mut(|path| path.push(chart_id))
        .tap_mut(|path| path.push("song.ogg"))
        .pipe(|path| StaticSoundData::from_file(path, StaticSoundSettings::default()))
        .map(|sound| KiraSource { sound })
    else {
        error!("Could not load audio file");
        return;
    };

    *song_info = SongInfo {
        dur: source.sound.duration().as_secs_f32().pipe(p32),
        pos: p32(start_from.raw() as f32),
        title: chart_id.to_string(),
        handle: song_channel
            .play(kira_sources.add(source))
            .start_from(start_from.raw())
            .looped()
            .handle(),
    };

    if matches!(state.0, GameState::Edit) {
        #[cfg(not(debug_assertions))]
        song_channel.pause();
    }
}

pub fn update_playback(mut song_info: ResMut<SongInfo>, instances: Res<Assets<KiraInstance>>) {
    song_info.pos = instances
        .get(&song_info.handle)
        .and_then(|instance| instance.state().position())
        .map_or(song_info.pos, |pos| p32(pos as f32))
}

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, game: &mut App) {
        game.add_plugin(KiraPlugin)
            .init_resource::<SongInfo>()
            .add_audio_channel::<SongChannel>()
            .add_event::<ChartLoadEvent>()
            .add_system(update_playback)
            .add_system(load_chart_song);
    }
}
