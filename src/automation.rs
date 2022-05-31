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
            Weight::Constant => t32(1.),
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
    start: T32,
    end: T32,
    weight: Weight,
}

impl RepeaterBound {
    fn eval(&self, t: T32) -> T32 {
        self.start.lerp(&self.end, self.weight.eval(t))
    }
}

struct Repeater {
    duration: R32,
    ceil: RepeaterBound,
    floor: RepeaterBound,
    repeat_bounds: bool,
}

#[derive(Default)]
struct ValueBound<T> {
    val: T,
    offset: R32,
}

impl<T> Quantify for ValueBound<T> {
    fn quantify(&self) -> R32 {
        self.offset
    }
}

struct Automation<T: Default> {
    start: R32,
    response: HitResponse,
    layer: u8,
    repeater: Option<Repeater>,
    upper_bounds: TinyVec<[ValueBound<T>; 4]>,
    anchors: TinyVec<[Anchor; 8]>,
    lower_bounds: TinyVec<[ValueBound<T>; 4]>,
}

impl<T: Copy + Default + Sample + Lerp> Automation<T> {
    #[rustfmt::skip]
    fn interp_anchors(anchors: &[Anchor], offset: R32) -> Option<T32> {
        let follow = anchors.before_or_at(offset).last();
        let control = anchors.after(offset).first();

        follow.zip(control).map(|(Anchor { point: follow, .. }, Anchor { point: control, weight })|
            t32(follow.y).lerp(
                &t32(control.y),
                weight.eval(offset.unit_interval(r32(follow.x), r32(control.x)))
            )
        )
    }

    fn sample_bound(bounds: &[ValueBound<T>], offset: R32) -> T {
        let control = bounds.before_or_at(offset).last().unwrap();
        let sampled = bounds.after(offset).first().map(|follow| {
            control.val.sample(
                &follow.val,
                offset.unit_interval(control.offset, follow.offset),
            )
        });

        sampled.unwrap_or(control.val)
    }

    fn eval(&self, offset: R32) -> Option<T> {
        let offset = offset - self.start;

        let options = match &self.repeater {
            Some(repeater) if offset < repeater.duration => {
                let period = r32(self.anchors.last().unwrap().point.x);
                let period_offset = offset % period;

                Self::interp_anchors(&self.anchors, self.start + period_offset).map(|lerp_amount| {
                    let repeat_bound_unit_interval = (offset / period)
                        .trunc()
                        .unit_interval(r32(0.), repeater.duration);

                    let bound_offset = match repeater.repeat_bounds {
                        true => period_offset,
                        false => offset,
                    };

                    let ceil = repeater.ceil.eval(repeat_bound_unit_interval);
                    let floor = repeater.floor.eval(repeat_bound_unit_interval);

                    (bound_offset, floor.lerp(&ceil, lerp_amount))
                })
            }
            _ => {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Needed for some constraints
    impl Sample for R32 {}

    #[test]
    fn weight_inflections() {
        assert_eq!(Weight::Constant.eval(t32(0.)), t32(1.));
        assert_eq!(Weight::Constant.eval(t32(0.5)), t32(1.));
        assert_eq!(Weight::Constant.eval(t32(1.)), t32(1.));
        assert_eq!(Weight::Quadratic(r32(0.)).eval(t32(0.5)), t32(0.5));
        (0..20).map(|i| i as f32).map(r32).for_each(|w| {
            assert_eq!(Weight::Quadratic(w).eval(t32(0.)), t32(0.));
            assert_eq!(Weight::Quadratic(w).eval(t32(1.)), t32(1.));
            assert_eq!(Weight::Cubic(w).eval(t32(0.)), t32(0.));
            assert_eq!(Weight::Cubic(w).eval(t32(0.5)), t32(0.5));
            assert_eq!(Weight::Cubic(w).eval(t32(1.)), t32(1.));
        })
    }

    #[test]
    fn anchor_interp() {
        let anchors = &[
            Anchor {
                point: Vec2::new(0., 0.),
                weight: Weight::Constant,
            },
            Anchor {
                point: Vec2::new(1., 1.),
                weight: Weight::Quadratic(r32(0.)),
            },
            Anchor {
                point: Vec2::new(2., 0.),
                weight: Weight::Quadratic(r32(0.)),
            },
        ];
        assert_eq!(
            Automation::<R32>::interp_anchors(anchors, r32(0.)),
            Some(t32(0.0))
        );
        assert_eq!(
            Automation::<R32>::interp_anchors(anchors, r32(0.5)),
            Some(t32(0.5))
        );
        assert_eq!(
            Automation::<R32>::interp_anchors(anchors, r32(1.0)),
            Some(t32(1.0))
        );
        assert_eq!(
            Automation::<R32>::interp_anchors(anchors, r32(1.5)),
            Some(t32(0.5))
        );
        assert_eq!(Automation::<R32>::interp_anchors(anchors, r32(2.)), None);
    }

    #[test]
    fn val_bound_sample() {
        let bounds = &[
            ValueBound {
                val: r32(0.),
                offset: r32(0.),
            },
            ValueBound {
                val: r32(1.),
                offset: r32(1.),
            },
        ];

        assert_eq!(Automation::<R32>::sample_bound(bounds, r32(0.)), r32(0.0));
        assert_eq!(Automation::<R32>::sample_bound(bounds, r32(0.5)), r32(0.0));
        assert_eq!(Automation::<R32>::sample_bound(bounds, r32(1.0)), r32(1.0));
        assert_eq!(Automation::<R32>::sample_bound(bounds, r32(2.0)), r32(1.0));
    }
}
