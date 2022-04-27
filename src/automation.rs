use bevy::prelude::*;
use noisy_float::prelude::*;
use tinyvec::TinyVec;

use crate::hit::*;
use crate::resources::*;
use crate::utils::*;

enum Weight {
    Constant,
    Quadratic(N32),
    Cubic(N32),
}

impl Default for Weight {
    fn default() -> Self {
        Weight::Constant
    }
}

#[derive(Default)]
struct Anchor {
    point: Vec2,
    weight: Weight,
}

impl Quantify for Anchor {
    fn quantify(&self) -> N32 {
        n32(self.point.x)
    }
}

impl Interpolate for Anchor {
    type Output = N32;
    fn interp(&self, other: &Self, t: N32) -> Self::Output {
        unimplemented!()
    }

    fn default(&self) -> Self::Output {
        n32(self.point.y)
    }
}

#[derive(Default)]
struct Bound<T> {
    val: T,
    offset: N32,
    transition: Weight,
}

struct Automation<T: Default> {
    start: N32,
    response: HitResponse,
    layer: u8,
    upper_bounds: TinyVec<[Bound<T>; 4]>,
    anchors: TinyVec<[Anchor; 4]>,
    lower_bounds: TinyVec<[Bound<T>; 4]>,
}

#[derive(Component)]
pub struct Channel<T: Default> {
    id: u8,
    clips: Vec<Automation<T>>,
}

impl<T: Default> Channel<T> {
    fn currently_skippable(&self, song_time: N32) -> bool {
        self.clips
            .last()
            .map_or(false, |clip| song_time < clip.start)
    }
}

#[derive(Component)]
pub struct IndexCache(usize);

fn seek_channels<T: Component + Default>(
    mut channel_table: Query<(&Channel<T>, &mut IndexCache)>,
    song_time: Res<SongTime>,
) {
    channel_table
        .iter_mut()
        .filter(|(channel, _)| !channel.currently_skippable(song_time.0))
        .for_each(|(channel, mut index_cache)| {
            if let Some(clip) = channel.clips.get(index_cache.0 + 1) {
                if clip.start <= song_time.0 {
                    index_cache.0 += 1;
                    return;
                }
            }

            index_cache.0 = channel
                .clips
                .as_slice()
                .seek(|automation| automation.start.cmp(&song_time.0))
        })
}

fn eval_channels<T: Component + Quantify + Interpolate + Default>(
    mut channel_table: Query<(&Channel<T>, &IndexCache)>,
    song_time: Res<SongTime>,
    mut output_table: ResMut<OutputTable<ChannelOutput<T>>>,
) {
    channel_table
        .iter_mut()
        .filter(|(channel, _)| !channel.clips.is_empty())
        .for_each(|(channel, cache)| {
            let clip = channel.clips[cache.0];
            let test = Some(clip.anchors.as_ref().interp(song_time.0 - clip.start));
            output_table.0[channel.id as usize] = unimplemented!();
        })
}

struct AutomationPlugin;

impl Plugin for AutomationPlugin {
    fn build(&self, app: &mut App) {}
}
