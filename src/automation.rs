use bevy::prelude::*;
use derive_more::{Deref, DerefMut};
use itertools::Itertools;
use noisy_float::prelude::*;
use tinyvec::TinyVec;

use crate::bounds::*;
use crate::hit::*;
use crate::resources::*;
use crate::utils::*;

#[derive(Debug, Clone, Copy)]
pub enum Weight {
    Constant,
    Quadratic(R32),
    Cubic(R32),
}

impl Weight {
    pub fn eval(&self, t: T32) -> T32 {
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
        t32(other.point.y).lerp(&t32(self.point.y), self.weight.eval(t))
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
    reaction: Option<(u8, HitReaction)>,
    repeater: Option<Repeater>,
    upper_bounds: TinyVec<[Bound<T>; 4]>,
    anchors: TinyVec<[Anchor; 8]>,
    lower_bounds: TinyVec<[Bound<T>; 4]>,
}

type AutomationOutput<T> = <<Bound<T> as Sample>::Output as Lerp>::Output;

pub struct ChannelOutput<T> {
    pub output: Option<T>,
    pub redirect: Option<usize>,
}

impl<T> Automation<T>
where
    T: Copy + Default + Quantify + Sample,
    <T as Sample>::Output: Lerp,
{
    #[rustfmt::skip]
    fn eval(&self, offset: R32) -> Option<AutomationOutput<T>> {
        self.repeater
            .as_ref()
            .and_then(|repeater| (offset < repeater.duration).then(|| {
                let period = r32(self.anchors.last().unwrap().point.x);
                let period_offset = offset % period;

                self.anchors.lerp(self.start + period_offset).map(|lerp_amount| {
                    let clamp_offset = (offset / period)
                        .trunc()
                        .unit_interval(r32(0.), repeater.duration);

                    let bound_offset = if repeater.repeat_bounds { period_offset } else { offset };
                    let lerp_amount = repeater
                        .floor
                        .eval(clamp_offset)
                        .lerp(&repeater.ceil.eval(clamp_offset), lerp_amount);

                    (bound_offset, lerp_amount)
                })
            }))
            .unwrap_or_else(|| self
                .anchors
                .lerp(offset)
                .map(|lerp_amount| (offset, lerp_amount))
            )
            .map(|(bound_offset, lerp_amount)| self
                .lower_bounds
                .sample(bound_offset)
                .lerp(&self.upper_bounds.sample(bound_offset), lerp_amount)
            )
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
    clips: Vec<Automation<T>>,
}

impl<T: Default> Channel<T> {
    fn can_skip_seeking(&self, song_time: R32) -> bool {
        self.clips
            .last()
            .map_or(true, |clip| clip.start < song_time)
    }
}

#[derive(Component, Deref, DerefMut)]
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
            **index_cache = channel
                .clips
                .iter()
                .enumerate()
                .skip(**index_cache)
                .coalesce(|prev, curr| (prev.1.start == curr.1.start)
                    .then(|| curr)
                    .ok_or((prev, curr))
                )
                .take(4)
                .take_while(|(_, clip)| clip.start < **song_time)
                .last()
                .map(|(index, _)| index)
                .unwrap_or_else(|| channel
                    .clips
                    .as_slice()
                    .seek(**song_time)
                )
        })
}

/// Envoke eval functions for each clip and juggle hit responses
fn eval_channels<T>(
    mut channel_table: Query<(&mut Channel<T>, &IndexCache)>,
    song_time: Res<SongTime>,
    hits: Res<HitRegister>,
    mut output_table: ResMut<AutomationOutputTable<AutomationOutput<T>>>,
) where
    T: Default + Copy + Component + Quantify + Sample,
    <T as Sample>::Output: Lerp,
    AutomationOutput<T>: Component,
{
    //
    //  TODO: Parallel system
    //
    channel_table
        .iter_mut()
        .filter(|(channel, _)| !channel.clips.is_empty())
        .for_each(|(mut channel, cache)| {
            let (slot, clip) = (
                &mut output_table[channel.id as usize],
                &mut channel.clips[**cache],
            );

            if let Some((layer, reaction)) = clip.reaction.as_mut() {
                hits.iter()
                    .filter_map(Option::as_ref)
                    .filter(|hit| hit.layer == *layer)
                    .for_each(|hit| reaction.react(hit));
            }

            let offset = **song_time - clip.start;

            slot.output = clip.eval(
                clip.reaction
                    .as_ref()
                    .map(|(_, reaction)| reaction.translate(offset))
                    .unwrap_or(offset),
            );

            slot.redirect = clip
                .reaction
                .as_ref()
                .and_then(|(_, reaction)| reaction.delegate())
                .map(From::from)
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

    #[test]
    fn weight_symmetry() {
        (-20..=-1)
            .chain(1..=20)
            .map(|i| i as f32)
            .map(r32)
            .for_each(|weight| {
                assert_ne!(Weight::Quadratic(weight).eval(t32(0.5)), t32(0.5));
                assert_ne!(Weight::Cubic(weight).eval(t32(0.25)), t32(0.25));
                assert_ne!(Weight::Cubic(weight).eval(t32(0.75)), t32(0.75));

                (1..50)
                    .chain(51..100)
                    .map(|i| i as f32)
                    .map(r32)
                    .map(|f| f.unit_interval(r32(0.), r32(100.)))
                    .for_each(|t| {
                        assert_eq!(
                            Weight::Quadratic(weight).eval(t) - Weight::Quadratic(weight).eval(t),
                            0.
                        );
                        assert_eq!(
                            Weight::Cubic(weight).eval(t) - Weight::Cubic(weight).eval(t),
                            0.
                        );
                    })
            })
    }

    #[test]
    fn weight_growth() {
        (-20..=20).map(|i| i as f32).map(r32).for_each(|weight| {
            (1..=100)
                .map(|i| i as f32)
                .map(r32)
                .map(|f| f.unit_interval(r32(0.), r32(100.)))
                .for_each(|t1| {
                    let t0 = t1 - 0.01;
                    assert!(
                        Weight::Quadratic(weight).eval(t0) < Weight::Quadratic(weight).eval(t1)
                    );
                    assert!(Weight::Cubic(weight).eval(t0) <= Weight::Cubic(weight).eval(t1));
                })
        })
    }

    fn automation() -> Automation<R32> {
        Automation {
            start: r32(0.),
            reaction: None,
            repeater: None,
            upper_bounds: tiny_vec![
                Bound {
                    value: r32(0.),
                    offset: r32(0.),
                },
                Bound {
                    value: r32(1.),
                    offset: r32(1.),
                },
                Bound {
                    value: r32(2.),
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
                Bound {
                    value: r32(0.),
                    offset: r32(0.),
                },
                Bound {
                    value: r32(1.),
                    offset: r32(1.),
                }
            ],
        }
    }

    #[test]
    fn anchor_interp() {
        let co_vals = [
            (0., Some(0.)),
            (0.5, Some(0.5)),
            (1.0, Some(1.0)),
            (1.5, Some(1.)),
            (2., Some(1.)),
            (3., None),
            (4., None),
            (5., None),
        ];

        co_vals
            .iter()
            .map(|&(input, output)| (r32(input), output.map(t32)))
            .for_each(|(input, output)| assert_eq!(automation().anchors.lerp(input), output));
    }

    #[test]
    fn automation_eval() {
        let co_vals = [
            (0., Some(0.)),
            (0.5, Some(0.)),
            (1., Some(1.)),
            (1.5, Some(1.)),
            (2.5, Some(1.5)),
        ];

        co_vals
            .iter()
            .map(|&(input, output)| (r32(input), output.map(r32)))
            .for_each(|(input, output)| assert_eq!(automation().eval(input), output));
    }
}
