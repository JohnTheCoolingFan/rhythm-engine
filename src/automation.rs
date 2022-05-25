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
    Quadratic(R32),
    Cubic(R32),
}

impl Weight {
    fn eval(&self, t: T32) -> T32 {
        let f = |x: f32, k: f32| x.signum() * x.abs().powf((k + k.signum()).abs().powf(k.signum()));

        match self {
            Weight::Constant => t32(0.),
            Weight::Quadratic(k) => t32(f(t.raw(), k.raw())),
            Weight::Cubic(k) => t32(((f(2. * t.raw() - 1., k.raw()) - 1.) / 2.) + 1.),
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
            weight: Weight::Quadratic(r32(0.)),
        }
    }
}

impl Quantify for Anchor {
    fn quantify(&self) -> R32 {
        r32(self.point.x)
    }
}

struct RepeaterBound {
    start: R32,
    end: R32,
    weight: Weight,
}

struct Repeater {
    duration: R32,
    ceil: RepeaterBound,
    floor: RepeaterBound,
    repeat_bounds: bool,
}

#[derive(Default)]
struct Bound<T> {
    val: T,
    offset: R32,
}

impl<T> Quantify for Bound<T> {
    fn quantify(&self) -> R32 {
        self.offset
    }
}

struct Automation<T: Default> {
    start: R32,
    response: HitResponse,
    layer: u8,
    repeater: Option<Repeater>,
    upper_bounds: TinyVec<[Bound<T>; 4]>,
    anchors: TinyVec<[Anchor; 8]>,
    lower_bounds: TinyVec<[Bound<T>; 4]>,
}

impl<T: Copy + Default + Sample + Lerp> Automation<T> {
    #[rustfmt::skip]
    fn interp_anchors(anchors: &[Anchor], offset: R32) -> Option<T32> {
        let follow = anchors.before_or_at(offset).last();
        let control = anchors.after(offset).first();

        follow.zip(control).map(|(follow, control)| 
            t32(
                follow.point.y
                + (control.point.y - control.point.y)
                * control
                    .weight
                    .eval(offset.unit_interval(r32(follow.point.x), r32(control.point.x)))
                    .raw()
            )
        )
    }

    fn sample_bound(bounds: &[Bound<T>], offset: R32) -> T {
        let control = bounds.before_or_at(offset).last().unwrap();
        let follow = bounds.after(offset).first().unwrap_or(control);

        control.val.sample(
            &follow.val,
            offset.unit_interval(control.offset, follow.offset),
        )
    }

    fn eval(&self, offset: R32) -> Option<T> {
        let options = match self.repeater {
            Some(repeater) if offset < repeater.duration + self.start => {
                let period = r32(self.anchors.last().unwrap().point.x);
                let anchor_offset = (offset - self.start) % period;
                
                Self::interp_anchors(&self.anchors, anchor_offset).map(|lerp_amount| {
                    let lerp_amount = 
                })
            }
            _ => {
                let offset = offset - self.start;
                Self::interp_anchors(&self.anchors, offset).map(|lerp_amount| (offset, lerp_amount))
            }
        };
        
        options.map(|(bound_offset, lerp_amount)| {
            let lower = Self::sample_bound(&self.lower_bounds, bound_offset);
            let upper = Self::sample_bound(&self.upper_bounds, bound_offset);
            lower.lerp(&upper, lerp_amount)
        })
    }
}

impl<T: Default> Quantify for Automation<T> {
    fn quantify(&self) -> R32 {
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
    fn can_skip_seeking(&self, song_time: R32) -> bool {
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
