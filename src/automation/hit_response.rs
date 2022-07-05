use bevy::prelude::*;
use noisy_float::prelude::*;

use crate::automation::*;
use crate::resources::*;

#[derive(Component)]
pub enum HitResponse {
    /// Stays at 0 state until hit, once hit which it will commece from the current time
    Commence,
    /// Switches to a different automation permenantly with a start from the current time
    Switch(u8),
    /// Switches to a different automation but will switch back to the original
    /// automation on another hit. This can be repeated indefinetly
    Toggle(u8),
    /// Will stay at 0 state with no hit, for each hit it will play the automation
    /// from the hit time to hit time + excess.
    Follow(R32),
}

#[derive(Component)]
pub enum ResponseState {
    Delegated(bool),
    Hit(R32),
    Empty,
}

fn respond_to_hits<T: Component>(
    mut channels: Query<(&Channel<T>, &IndexCache, &mut ResponseState)>,
    clips: Query<&HitResponse>,
    song_time: Res<SongTime>,
    hits: Res<HitRegister>,
) {
    channels
        .iter_mut()
        .filter(|(_, index_cache, _)| matches!(index_cache, IndexCache::Dirty(_)))
        .for_each(|(_, _, mut response_state)| *response_state = ResponseState::Empty);

    channels
        .iter_mut()
        .filter(|(_, index_cache, _)| matches!(index_cache, IndexCache::Clean(_)))
        .filter_map(|(channel, index_cache, response_state)| {
            clips
                .get(channel.data[index_cache.get()].entity)
                .ok()
                .map(|clip| (clip, response_state))
        });
}
