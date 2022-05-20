use bevy::prelude::*;
use itertools::Itertools;
use noisy_float::prelude::*;
use tinyvec::TinyVec;

use crate::hit::*;
use crate::resources::*;
use crate::utils::*;

#[derive(Debug, Clone, Copy)]
enum Weight {
    Constant,
    Quadratic(N32),
    Cubic(N32),
}

impl Weight {
    fn eval(&self, t: N32) -> N32 {
        let f = |x: f32, k: f32| x.signum() * x.abs().powf((k + k.signum()).abs().powf(k.signum()));

        match self {
            Weight::Constant => n32(0.),
            Weight::Quadratic(k) => n32(f(t.raw(), k.raw())),
            Weight::Cubic(k) => n32(((f(2. * t.raw() - 1., k.raw()) - 1.) / 2.) + 1.),
        }
    }
}

struct Anchor {
    point: Vec2,
    weight: Weight,
}

impl Default for Anchor {
    fn default() -> Self {
        Anchor {
            point: Vec2::default(),
            weight: Weight::Quadratic(n32(0.)),
        }
    }
}

impl Quantify for Anchor {
    fn quantify(&self) -> N32 {
        n32(self.point.x)
    }
}

struct RepeaterBound {
    start: N32,
    end: N32,
    weight: Weight,
}

struct Repeater {
    duration: N32,
    ceil: RepeaterBound,
    floor: RepeaterBound,
}

#[derive(Default)]
struct Bound<T> {
    val: T,
    offset: N32,
}

impl<T> Quantify for Bound<T> {
    fn quantify(&self) -> N32 {
        self.offset
    }
}

struct Automation<T: Default> {
    start: N32,
    response: HitResponse,
    layer: u8,
    repeater: Option<Repeater>,
    upper_bounds: TinyVec<[Bound<T>; 4]>,
    anchors: TinyVec<[Anchor; 8]>,
    lower_bounds: TinyVec<[Bound<T>; 4]>,
}

impl<T: Default + Sample> Automation<T> {
    #[rustfmt::skip]
    fn eval(&self, t: N32) -> Option<T> {
        let interp = |t: N32| {
            self.anchors.as_slice().after(t).first().and_then(|Anchor { point: control, weight }| {
                self.anchors.as_slice().before_or_at(t).last().map(|Anchor { point: follow, .. }| {
                    let t = (t - follow.x) / (control.x - follow.x);
                    n32(follow.y) + n32(control.y - follow.y) * weight.eval(t)
                })
            })
        };

        let t = t - self.start;

        match &self.repeater {
            Some(repeater) => unimplemented!(),
            None => unimplemented!(),
        }
    }
}

impl<T: Default> Quantify for Automation<T> {
    fn quantify(&self) -> N32 {
        self.start
    }
}

#[derive(Component)]
pub struct Channel<T: Default> {
    id: u8,
    /// Evals by last (<= t)
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
fn seek_channels<T: Default + Component>(
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

fn eval_channels<T: Default + Component + Quantify>(
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
