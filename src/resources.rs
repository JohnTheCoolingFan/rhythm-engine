use noisy_float::prelude::*;

use crate::{automation::ChannelOutput, hit::HitInfo, spline::Displacement};

pub const CHANNELS_PER_TABLE: usize = 256;

pub struct HitRegister(pub [Option<HitInfo>; 4]);

pub struct SongTime(pub R32);

pub struct AutomationOutputTable<T>(pub [ChannelOutput<T>; CHANNELS_PER_TABLE]);

pub struct SplineOutputTable(pub [Displacement; CHANNELS_PER_TABLE]);
