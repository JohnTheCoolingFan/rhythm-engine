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

#[derive(Component, Deref, DerefMut)]
struct ChannelCache(usize);

pub trait AutomationClip {
    type Output;
    type ClipCache: Component;
    fn play(&self, clip_time: R32, cache: &mut Self::ClipCache) -> Self::Output;
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
        &mut ChannelCache,
        &mut ResponseState,
        &mut <T as AutomationClip>::ClipCache,
        &mut OutputSlot<<T as AutomationClip>::Output>,
    )>,
    clips: Query<(
        &T,
        Option<&HitResponse>,
    )>,
)
where
    T: Default + Component + AutomationClip,
    <T as AutomationClip>::Output: Send + Sync,
    <T as AutomationClip>::ClipCache: Default + Component
{
    table.iter_mut().for_each(|(channel, mut index, mut response_state, mut clip_cache, mut slot)| {
        let hits: &[Option<HitInfo>];
        let old_index = **index;

        if !channel.data.can_skip_reindex(**song_time) {
            **index = channel.data.reindex_through(**song_time, **index);
        }

        //  Might need more complex hit connection logic here
        if **index != old_index {
            *response_state = ResponseState::Nil;
            *clip_cache = Default::default();
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

        slot.output = clip.play(clip_time, &mut *clip_cache);
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use Weight::*;

    #[test]
    fn weight_inflections() {
        assert_eq!(Constant.eval(t32(0.)), t32(1.));
        assert_eq!(Constant.eval(t32(0.5)), t32(1.));
        assert_eq!(Constant.eval(t32(1.)), t32(1.));
        assert_eq!(Quadratic(r32(0.)).eval(t32(0.5)), t32(0.5));

        (-20..20).map(|i| i as f32).map(r32).for_each(|weight| {
            assert_eq!(Quadratic(weight).eval(t32(0.)), t32(0.));
            assert_eq!(Quadratic(weight).eval(t32(1.)), t32(1.));
            assert_eq!(Cubic(weight).eval(t32(0.)), t32(0.));
            assert_eq!(Cubic(weight).eval(t32(0.5)), t32(0.5));
            assert_eq!(Cubic(weight).eval(t32(1.)), t32(1.));
        })
    }

    #[test]
    #[rustfmt::skip]
    fn weight_symmetry() {
        (-20..=-1).chain(1..=20).map(|i| i as f32).map(r32).for_each(|weight| {
            assert_ne!(Quadratic(weight).eval(t32(0.5)), t32(0.5));
            assert_ne!(Cubic(weight).eval(t32(0.25)), t32(0.25));
            assert_ne!(Cubic(weight).eval(t32(0.75)), t32(0.75));

            (1..50).chain(51..100).map(|i| t32((i as f32) / 100.)).for_each(|t| {
                assert_eq!(Quadratic(weight).eval(t) - Quadratic(weight).eval(t), 0.);
                assert_eq!(Cubic(weight).eval(t) - Cubic(weight).eval(t), 0.);
            })
        })
    }

    #[test]
    fn weight_growth() {
        (-20..=20).map(|i| i as f32).map(r32).for_each(|weight| {
            (1..=100).map(|i| t32((i as f32) / 100.)).for_each(|t1| {
                let t0 = t1 - 0.01;
                assert!(Quadratic(weight).eval(t0) < Quadratic(weight).eval(t1));
                assert!(Cubic(weight).eval(t0) <= Cubic(weight).eval(t1));
            })
        })
    }
}
