use derive_more::{Deref, DerefMut};
use noisy_float::prelude::*;

use crate::{automation::ChannelOutput, hit::HitInfo, spline::*};

pub const CHANNELS_PER_AUTOMATION_TABLE: usize = u8::MAX as usize + 1;

#[derive(Deref, DerefMut)]
pub struct HitRegister(pub [Option<HitInfo>; 4]);

#[derive(Deref, DerefMut)]
pub struct SongTime(pub R32);

#[derive(Deref, DerefMut)]
pub struct AutomationOutputTable<T>(pub [ChannelOutput<T>; CHANNELS_PER_AUTOMATION_TABLE]);

pub const MAX_SPLINES: usize = u16::MAX as usize + 1;

#[derive(Deref, DerefMut)]
pub struct SplineTable(pub [Option<(Spline, SplineLut)>; MAX_SPLINES]);
