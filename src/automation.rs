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

mod test {}

impl Default for Weight {
    fn default() -> Self {
        Self::Constant
    }
}

struct RepeaterBound {
    start_y: N32,
    end_y: N32,
    transition: Weight,
}

enum Anchor {
    ControlPoint {
        point: Vec2,
        weight: Weight,
    },
    Repeater {
        point: Vec2,
        take_size: usize,
        roof: RepeaterBound,
        floor: RepeaterBound,
    },
}

impl Anchor {
    fn eval(&self, passed: &[Self], t: N32) -> Option<N32> {}
}

impl Default for Anchor {
    fn default() -> Self {
        Self::ControlPoint {
            point: Vec2::new(0.0, 0.0),
            weight: Weight::Constant,
        }
    }
}

impl Quantify for Anchor {
    fn quantify(&self) -> N32 {
        match self {
            Self::ControlPoint { point, .. } => n32(point.x),
            Self::Repeater { point, .. } => n32(point.x),
        }
    }
}

#[derive(Default)]
struct Bound<T> {
    val: T,
    offset: N32,
    transition: Weight,
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
    /// Evals by last (<= t)
    upper_bounds: TinyVec<[Bound<T>; 4]>,
    /// Evals by first (t <)
    anchors: TinyVec<[Anchor; 4]>,
    /// Evals by last (<= t)
    lower_bounds: TinyVec<[Bound<T>; 4]>,
}

impl<T: Default + Copy + Lerp> Automation<T> {
    fn eval(self, t: N32) -> Option<T> {
        let passed = self
            .anchors
            .iter()
            .take_while(|item| item.quantify() <= t)
            .count();

        let interpolate = |bounds: &[Bound<T>]| {
            let before = bounds.first_before(t).unwrap();
            bounds.first_after(t).map_or(before.val, |after| {
                let t = (t - before.quantify()) / (after.quantify() - before.quantify());
                let d = before.transition.eval(t);
                before.val.lerp(after.val, d)
            })
        };

        self.anchors.as_slice().first_after(t).and_then(|after| {
            after.eval(&self.anchors[..passed], t).map(|t| {
                let lower = interpolate(&self.lower_bounds);
                let upper = interpolate(&self.upper_bounds);
                lower.lerp(upper, t)
            })
        })
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
