mod bound_sequence;
mod hit_response;
mod spline;

use hit_response::*;

use std::marker::PhantomData;

use bevy::prelude::*;
use noisy_float::prelude::*;

use crate::{hit::*, resources::*, utils::*};

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

impl Default for Weight {
    fn default() -> Self {
        Self::Quadratic(r32(0.))
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct ClipID(u32);

#[derive(Clone, Copy)]
pub struct ClipInstance {
    offset: R32,
    entity: Entity,
}

impl Quantify for ClipInstance {
    fn quantify(&self) -> R32 {
        self.offset
    }
}

#[derive(Default, Clone, Copy, Component)]
pub struct ChannelID(u8);

#[derive(Default, Component)]
pub struct Channel<T> {
    pub data: Vec<ClipInstance>,
    _phantom: PhantomData<T>,
}

impl<T> ControllerTable for Channel<T> {
    type Item = ClipInstance;
    fn table(&self) -> &[Self::Item] {
        self.data.as_slice()
    }
}

#[derive(Component, Deref, DerefMut)]
struct IndexCache(usize);

pub trait AutomationClip {
    type Output;
    fn play(&self, clip_time: R32) -> Self::Output;
}

#[derive(Component)]
pub struct OutputSlot<T> {
    output: T,
    redirect: Option<u8>,
}

#[rustfmt::skip]
fn eval_automation_table<T>(
    song_time: Res<SongTime>,
    hit_reg: Res<HitRegister>,
    mut table: Query<(
        &Channel<T>,
        &mut IndexCache,
        &mut ResponseState,
        &mut OutputSlot<<T as AutomationClip>::Output>
    )>,
    clips: Query<(
        &T,
        Option<&HitResponse>,
    )>,
)
where
    T: Default + Component + AutomationClip,
    <T as AutomationClip>::Output: Send + Sync
{
    table.iter_mut().for_each(|(channel, mut index, mut response_state, mut slot)| {
        let hits: &[Option<HitInfo>];
        let old_index = **index;

        if !channel.can_skip_reindex(**song_time) {
            **index = channel.reindex_through(**song_time, **index);
        }

        if **index != old_index {
            *response_state = ResponseState::Nil;
            hits = &[];
        } else {
            hits = &**hit_reg;
        }

        let ((clip, response), clip_start) = channel
            .data
            .get(**index)
            .map(|instance| (clips.get(instance.entity).unwrap(), instance.offset))
            .unwrap();

        let clip_time: R32;

        if let Some(&HitResponse{ kind, layer }) = response.as_ref() {
            use ResponseKind::*;
            use ResponseState::*;

            hits.iter().flatten().filter(|hit| hit.layer == *layer).for_each(|hit|
                match (kind, &mut *response_state) {
                    (Commence | Switch(_), state) => *state = Delegated(true),
                    (Toggle(_), Delegated(delegate)) => *delegate = !*delegate,
                    (Toggle(_), state) => *state = Delegated(true),
                    (Follow(_), last_hit) => *last_hit = Hit(hit.object_time),
                    _ => {}
                }
            );

            slot.redirect = match (kind, &mut *response_state) {
                (Switch(target) | Toggle(target), Delegated(true)) => Some(*target),
                _ => None
            };

            clip_time = (- clip_start) + match (kind, &mut *response_state) {
                (Commence, Delegated(delegate)) if !*delegate => clip_start,
                (Follow(ex), Hit(hit)) if !(*hit..*hit + ex).contains(&**song_time) => *hit + ex,
                _ => **song_time
            };
        } else {
            slot.redirect = None;
            clip_time = **song_time - clip_start;
        }

        slot.output = clip.play(clip_time);
    })
}

/*type AutomationOutput<T> = <<T as Lerp>::Output as Lerp>::Output;

pub struct ChannelOutput<T> {
    pub output: Option<T>,
    pub redirect: Option<usize>,
}

impl<T> Automation<T>
where
    T: Default + Quantify + Lerp,
    <T as Lerp>::Output: Lerp<Output = <T as Lerp>::Output>,
{
    #[rustfmt::skip]
    fn eval(&self, offset: R32) -> Option<AutomationOutput<T>> {
        self.repeater
            .as_ref()
            .and_then(|Repeater { duration, floor, ceil, repeat_bounds }| {
                (offset < *duration).then(|| {
                    let period = r32(self.anchors.last().unwrap().point.x);
                    let period_offset = offset % period;

                    self.anchors.interp(self.start + period_offset).map(|lerp_amount| {
                        let clamp_offset = (offset / period)
                            .trunc()
                            .unit_interval(r32(0.), *duration);
                        let lerp_amount = floor
                            .eval(clamp_offset)
                            .lerp(&ceil.eval(clamp_offset), lerp_amount);

                        (if *repeat_bounds { period_offset } else { offset }, lerp_amount)
                    })
                })
            })
            .unwrap_or_else(|| self
                .anchors
                .interp(offset)
                .map(|lerp_amount| (offset, lerp_amount))
            )
            .map(|(bound_offset, lerp_amount)| self
                .lower_bounds
                .interp_or_last(bound_offset)
                .lerp(&self.upper_bounds.interp_or_last(bound_offset), lerp_amount)
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
    fn can_skip(&self, offset: R32) -> bool {
        self.clips
            .last()
            .map_or(true, |item| item.quantify() < offset)
    }
}

impl<T: Default + Quantify> ControllerTable for Channel<T> {
    type Item = Automation<T>;
    fn table(&self) -> &[Self::Item] {
        self.clips.as_slice()
    }
}

#[derive(Component)]
pub struct ChannelSeeker {
    index_cache: usize,
    reaction_state: ReactionState,
}

/// Envoke eval functions for each clip and juggle hit responses
#[rustfmt::skip]
fn eval_channel_primaries<T>(
    mut channel_table: Query<(&mut Channel<T>, &mut ChannelSeeker)>,
    song_time: Res<SongTime>,
    hit_reg: Res<HitRegister>,
    mut primary_output_table: ResMut<AutomationOutputTable<AutomationOutput<T>>>,
) where
    T: Default + Component + Quantify + Lerp,
    <T as Lerp>::Output: Lerp<Output = <T as Lerp>::Output>,
    AutomationOutput<T>: Component,
{
    //
    //  TODO: Parallel system
    //
    channel_table
        .iter_mut()
        .filter(|(channel, _)| channel.can_skip(**song_time))
        .for_each(|(mut channel, mut seeker)| {
            let hits: &[Option<HitInfo>];
            let new_index = channel.find_index_through(**song_time, seeker.index_cache);

            if new_index != seeker.index_cache {
                seeker.index_cache = new_index;
                seeker.reaction_state = ReactionState::Empty;
                hits = &[];
            } else {
                hits = &**hit_reg;
            }

            let (slot, clip, reaction_state) = (
                &mut primary_output_table[channel.id as usize],
                &mut channel.clips[seeker.index_cache],
                &mut seeker.reaction_state
            );

            use HitReaction::*;
            use ReactionState::*;

            if let Some((layer, reaction)) = clip.reaction.as_ref() {
                hits.iter().flatten().filter(|hit| hit.layer == *layer).for_each(|hit|
                    match (reaction, &mut *reaction_state) {
                        (Commence | Switch(_), state) => *state = Delegated(true),
                        (Toggle(_), Delegated(delegate)) => *delegate = !*delegate,
                        (Toggle(_), state) => *state = Delegated(true),
                        (Follow(_), last_hit) => *last_hit = Hit(hit.object_time)
                    }
                )
            }

            let offset = **song_time - clip.start;

            slot.output = clip.eval(
                clip.reaction.as_ref().map_or(offset, |(_, reaction)|
                    match (reaction, &mut *reaction_state) {
                        (Commence, Delegated(delegate)) => if *delegate { offset } else { r32(0.) },
                        (Follow(ex), Hit(hit)) if !(*hit..*hit + ex).contains(&offset) => *hit + ex,
                        _ => offset
                    }
                )
            );

            slot.redirect = clip.reaction.as_ref().and_then(|(_, reaction)|
                match (reaction, &mut *reaction_state) {
                    (Switch(target) | Toggle(target), Delegated(true)) => Some(*target as usize),
                    _ => None
                }
            )
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
    use crate::bounds::*;
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

    fn automation() -> Automation<ScalarBound<R32>> {
        Automation {
            start: r32(0.),
            reaction: None,
            repeater: None,
            upper_bounds: tiny_vec![
                ScalarBound {
                    scalar: r32(0.),
                    offset: r32(0.),
                },
                ScalarBound {
                    scalar: r32(1.),
                    offset: r32(1.),
                },
                ScalarBound {
                    scalar: r32(2.),
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
                ScalarBound {
                    scalar: r32(0.),
                    offset: r32(0.),
                },
                ScalarBound {
                    scalar: r32(1.),
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
            .for_each(|(input, output)| assert_eq!(automation().anchors.interp(input), output));
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
}*/
