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

impl Lerp for Anchor {
    type Output = T32;
    fn lerp(&self, other: &Self, t: T32) -> Self::Output {
        t32(other.point.y).lerp(&t32(self.point.y), t)
    }
}

struct RepeaterClamp {
    start: T32,
    end: T32,
    weight: Weight,
}

impl RepeaterClamp {
    fn eval(&self, t: T32) -> T32 {
        self.start.lerp(&self.end, self.weight.eval(t))
    }
}

struct Repeater {
    duration: R32,
    ceil: RepeaterClamp,
    floor: RepeaterClamp,
    repeat_bounds: bool,
}

struct Automation<T: Default> {
    start: R32,
    reaction: HitReaction,
    layer: Option<u8>,
    repeater: Option<Repeater>,
    upper_bounds: TinyVec<[T; 4]>,
    anchors: TinyVec<[Anchor; 8]>,
    lower_bounds: TinyVec<[T; 4]>,
}

type AutomationOutput<T> = <<T as Sample>::Output as Lerp>::Output;

impl<T> Automation<T>
where
    T: Copy + Default + Quantify + Sample + Lerp,
    <T as Sample>::Output: Lerp,
{
    #[rustfmt::skip]
    fn eval(&self, hits: &HitRegister, offset: R32) -> (Option<u8>, Option<AutomationOutput<T>>) {
        let (delegate, offset) = self.reaction.react(hits, offset - self.start);

        let options = match &self.repeater {
            Some(repeater) if offset < repeater.duration => {
                let period = r32(self.anchors.last().unwrap().point.x);
                let period_offset = offset % period;

                self.anchors.lerp(self.start + period_offset).map(|lerp_amount| {
                    let bound_offset = if repeater.repeat_bounds { period_offset } else { offset };
                    let clamp_offset = (offset / period)
                        .trunc()
                        .unit_interval(r32(0.), repeater.duration);

                    let (floor, ceil) = (
                        repeater.floor.eval(clamp_offset),
                        repeater.ceil.eval(clamp_offset),
                    );

                    (bound_offset, floor.lerp(&ceil, lerp_amount))
                })
            }
            _ => {
                self.anchors.lerp(offset).map(|lerp_amount| (offset, lerp_amount))
            }
        };

        let output = options.map(|(bound_offset, lerp_amount)| {
            let (lower, upper) = (
                self.lower_bounds.sample(bound_offset),
                self.upper_bounds.sample(bound_offset),
            );

            lower.lerp(&upper, lerp_amount)
        });

        (delegate, output)
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

/// Find each clip we want to evaluate on in each channel
#[rustfmt::skip]
fn seek_channels<T: Default + Component>(
    mut channel_table: Query<(&Channel<T>, &mut IndexCache)>,
    song_time: Res<SongTime>,
) {
    //
    //  TODO: Parallel system
    //
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

/// Envoke eval functions for each clip and juggle hit responses
fn eval_channels<T>(
    channel_table: Query<(&Channel<T>, &IndexCache)>,
    song_time: Res<SongTime>,
    hit_reg: Res<HitRegister>,
    mut output_table: ResMut<OutputTable<AutomationOutput<T>>>,
) where
    T: Default + Copy + Component + Quantify + Sample + Lerp,
    <T as Sample>::Output: Lerp,
    AutomationOutput<T>: Component,
{
    //
    //  TODO: Parallel system
    //
    channel_table
        .iter()
        .filter(|(channel, _)| !channel.clips.is_empty())
        .for_each(|(channel, cache)| {
            let (slot, clip) = (
                &mut output_table.0[channel.id as usize],
                &channel.clips[cache.0],
            );

            let (delegate, output) = clip.eval(&hit_reg, song_time.0);

            slot.output = output;
            slot.redirect = delegate.map(From::from);
        })
}

struct AutomationPlugin;

impl Plugin for AutomationPlugin {
    fn build(&self, app: &mut App) {}
}

#[cfg(test)]
mod tests {
    //
    //  TODO:
    //          - Test Repeater
    //          - Test HitReactions
    //
    use super::*;
    use tinyvec::tiny_vec;

    /// Needed for some constraints
    impl Sample for R32 {}

    #[test]
    fn weight_inflections() {
        assert_eq!(Weight::Constant.eval(t32(0.)), t32(1.));
        assert_eq!(Weight::Constant.eval(t32(0.5)), t32(1.));
        assert_eq!(Weight::Constant.eval(t32(1.)), t32(1.));
        assert_eq!(Weight::Quadratic(r32(0.)).eval(t32(0.5)), t32(0.5));

        (-20..20).map(|i| i as f32).map(r32).for_each(|weight| {
            assert_eq!(Weight::Quadratic(weight).eval(t32(0.)), t32(0.));
            assert_eq!(Weight::Quadratic(weight).eval(t32(1.)), t32(1.));
            assert_eq!(Weight::Cubic(weight).eval(t32(0.)), t32(0.));
            assert_eq!(Weight::Cubic(weight).eval(t32(0.5)), t32(0.5));
            assert_eq!(Weight::Cubic(weight).eval(t32(1.)), t32(1.));
        })
    }

    fn automation() -> Automation<R32> {
        Automation::<R32> {
            start: r32(0.),
            reaction: HitReaction::Ignore,
            layer: None,
            repeater: None,
            upper_bounds: tiny_vec![
                ValueBound {
                    val: r32(0.),
                    offset: r32(0.),
                },
                ValueBound {
                    val: r32(1.),
                    offset: r32(1.),
                },
                ValueBound {
                    val: r32(2.),
                    offset: r32(2.),
                }
            ],
            anchors: tiny_vec![
                Anchor {
                    point: Vec2::new(0., 0.),
                    weight: Weight::Constant,
                },
                Anchor {
                    point: Vec2::new(1., 1.),
                    weight: Weight::Quadratic(r32(0.)),
                },
                Anchor {
                    point: Vec2::new(2., 1.),
                    weight: Weight::Quadratic(r32(0.)),
                },
                Anchor {
                    point: Vec2::new(3., 0.),
                    weight: Weight::Quadratic(r32(0.)),
                }
            ],
            lower_bounds: tiny_vec![
                ValueBound {
                    val: r32(0.),
                    offset: r32(0.),
                },
                ValueBound {
                    val: r32(1.),
                    offset: r32(1.),
                }
            ],
        }
    }

    #[test]
    fn anchor_interp() {
        assert_eq!(automation().anchors.lerp(r32(0.)), Some(t32(0.0)));
        assert_eq!(automation().anchors.lerp(r32(0.5)), Some(t32(0.5)));
        assert_eq!(automation().anchors.lerp(r32(1.0)), Some(t32(1.0)));
        assert_eq!(automation().anchors.lerp(r32(1.5)), Some(t32(1.)));
        assert_eq!(automation().anchors.lerp(r32(2.)), Some(t32(1.)));
        assert_eq!(automation().anchors.lerp(r32(3.)), None);
        assert_eq!(automation().anchors.lerp(r32(4.)), None);
        assert_eq!(automation().anchors.lerp(r32(5.)), None);
    }

    #[test]
    fn val_bound_sample() {
        assert_eq!(
            Automation::<R32>::sample_bound(&automation().lower_bounds, r32(0.)),
            r32(0.0)
        );
        assert_eq!(
            Automation::<R32>::sample_bound(&automation().lower_bounds, r32(0.5)),
            r32(0.0)
        );
        assert_eq!(
            Automation::<R32>::sample_bound(&automation().lower_bounds, r32(1.0)),
            r32(1.0)
        );
        assert_eq!(
            Automation::<R32>::sample_bound(&automation().lower_bounds, r32(2.0)),
            r32(1.0)
        );
        assert_eq!(
            Automation::<R32>::sample_bound(&automation().lower_bounds, r32(3.0)),
            r32(1.0)
        );
        assert_eq!(
            Automation::<R32>::sample_bound(&automation().lower_bounds, r32(4.0)),
            r32(1.0)
        );
    }

    #[test]
    fn automation_eval() {
        assert_eq!(
            automation().eval(&HitRegister([None; 4]), r32(0.)),
            (None, Some(r32(0.)))
        );
        assert_eq!(
            automation().eval(&HitRegister([None; 4]), r32(0.5)),
            (None, Some(r32(0.)))
        );
        assert_eq!(
            automation().eval(&HitRegister([None; 4]), r32(1.)),
            (None, Some(r32(1.)))
        );
        assert_eq!(
            automation().eval(&HitRegister([None; 4]), r32(1.5)),
            (None, Some(r32(1.)))
        );
        assert_eq!(
            automation().eval(&HitRegister([None; 4]), r32(2.5)),
            (None, Some(r32(1.5)))
        );
    }
}
