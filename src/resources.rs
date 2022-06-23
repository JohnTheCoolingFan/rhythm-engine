use derive_more::{Deref, DerefMut};
use noisy_float::prelude::*;

use crate::{automation::ChannelOutput, hit::HitInfo, spline::SplineLut};

pub const CHANNELS_PER_AUTOMATION_TABLE: usize = 256;

#[derive(Deref, DerefMut)]
pub struct HitRegister(pub [Option<HitInfo>; 4]);

#[derive(Deref, DerefMut)]
pub struct SongTime(pub R32);

#[derive(Deref, DerefMut)]
pub struct AutomationOutputTable<T>(pub [ChannelOutput<T>; CHANNELS_PER_AUTOMATION_TABLE]);

#[derive(Deref, DerefMut)]
pub struct SplineLuts(pub Vec<SplineLut>);
