use bevy::prelude::*;
use noisy_float::prelude::*;

use crate::automation::*;
use crate::resources::*;

#[derive(Component)]
pub enum ResponseKind {
    Nil,
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
pub struct HitResponse {
    pub kind: ResponseKind,
    pub layer: u8,
}

#[derive(Component)]
pub enum ResponseState {
    Nil,
    Hit(R32),
    Delegated(bool),
}

#[rustfmt::skip]
fn respond_to_hits<T: Component>(
    In(instances): In<impl Iterator<Item = (ChannelID, Watched<Entity>, R32)>>,
    //clips: Query<(&HitResponse, &ResponseLayer)>,
    song_time: Res<SongTime>,
    hits: Res<HitRegister>,
) {
    /*channels
        .iter_mut()
        .filter(|(_, channel, _)| channel.can_skip(**song_time))
        .for_each(|(index_cache, channel, mut response_state)| {
            if let IndexCache::Dirty(_) = index_cache {
                *response_state = ResponseState::Empty
            } else {
                let instance = channel.data[**index_cache];
                let (response, layer) = clips.get(instance.entity).unwrap();

                use HitResponse::*;
                use ResponseState::*;

                hits.iter().flatten().filter(|hit| hit.layer == **layer).for_each(|hit|
                    match (response, &mut *response_state) {
                        (Commence | Switch(_), state) => *state = Delegated(true),
                        (Toggle(_), Delegated(delegate)) => *delegate = !*delegate,
                        (Toggle(_), state) => *state = Delegated(true),
                        (Follow(_), last_hit) => *last_hit = Hit(hit.object_time),
                    }
                )
            }
        }
    )*/
}

/*#[rustfmt::skip]
fn process_hits<'a>(
    hits: Res<'a, HitRegister>,
    clips: Query<'a, 'a, (&HitResponse, &ResponseLayer)>
)
    -> impl Fn((Watched<Entity>, R32), &mut ResponseState) -> (Entity, R32, Redirect) + 'a
{
    move |(instance, offset), response_state| {
        let (response, layer) = clips.get(*instance).unwrap();

        use HitResponse::*;
        use ResponseState::*;
        hits.iter().flatten().filter(|hit| hit.layer == **layer).for_each(|hit| {
            match (response, &mut *response_state) {
                (Commence | Switch(_), state) => *state = Delegated(true),
                (Toggle(_), Delegated(delegate)) => *delegate = !*delegate,
                (Toggle(_), state) => *state = Delegated(true),
                (Follow(_), last_hit) => *last_hit = Hit(hit.object_time),
            }
        });

        todo!()
    }
}*/
