use bevy::prelude::*;
use itertools::Itertools;
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

struct ControlPoint {
    point: Vec2,
    weight: Weight,
}

struct RepeaterBound {
    start_y: N32,
    end_y: N32,
    transition: Weight,
}

struct Repeater {
    offset: N32,
    take_size: usize,
    roof: RepeaterBound,
    floor: RepeaterBound,
}

enum Anchor {
    ControlPoint(ControlPoint),
    Repeater(Repeater),
}

impl Default for Anchor {
    fn default() -> Self {
        Anchor::ControlPoint(ControlPoint {
            point: Vec2::new(0.0, 0.0),
            weight: Weight::Constant,
        })
    }
}

impl Quantify for Anchor {
    fn quantify(&self) -> N32 {
        match self {
            Self::ControlPoint(ControlPoint { point, .. }) => n32(point.x),
            Self::Repeater(Repeater { offset, .. }) => *offset,
        }
    }
}

impl Interpolate for Anchor {
    type Output = Option<N32>;
    fn interp(&self, passed: &[Self], t: N32) -> Self::Output {
        match (passed, self) {
            ([.., last] | [last], Self::ControlPoint { point, weight }) => {
                unimplemented!()
            }
            (passed, Self::Repeater { offset, step_size }) if *step_size <= passed.len() => {
                unimplemented!()
            }
            _ => None,
        }
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

impl<T: Default> Quantify for Automation<T> {
    fn quantify(&self) -> N32 {
        self.start
    }
}

#[derive(Component)]
pub struct Channel<T: Default> {
    id: u8,
    clips: Vec<Automation<T>>,
}

impl<T: Default> Channel<T> {
    fn can_skip_seeking(&self, song_time: N32) -> bool {
        self.clips
            .last()
            .map_or(true, |clip| clip.start < song_time)
    }
}

#[derive(Component)]
pub struct IndexCache(usize);

#[rustfmt::skip]
fn seek_channels<T: Component + Default>(
    mut channel_table: Query<(&Channel<T>, &mut IndexCache)>,
    song_time: Res<SongTime>,
) {
    channel_table
        .iter_mut()
        .filter(|(channel, _)| !channel.can_skip_seeking(song_time.0))
        .for_each(|(channel, mut index_cache)| {
            index_cache.0 = channel
                .clips
                .iter()
                .enumerate()
                .skip(index_cache.0)
                .coalesce(|prev, curr| (prev.1.start == curr.1.start)
                    .then(|| curr)
                    .ok_or((prev, curr))
                )
                .take(4)
                .take_while(|(_, clip)| clip.start < song_time.0)
                .last()
                .map(|(index, _)| index)
                .unwrap_or_else(|| channel
                    .clips
                    .as_slice()
                    .seek(song_time.0)
                )
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
        .for_each(|(channel, cache)| unimplemented!())
}

struct AutomationPlugin;

impl Plugin for AutomationPlugin {
    fn build(&self, app: &mut App) {}
}
