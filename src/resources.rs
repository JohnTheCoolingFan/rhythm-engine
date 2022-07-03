use derive_more::{Deref, DerefMut};
use noisy_float::prelude::*;

use crate::{automation::ChannelOutput, hit::HitInfo};

pub const CHANNELS_PER_AUTOMATION_TABLE: usize = u8::MAX as usize + 1;

#[derive(Deref, DerefMut)]
pub struct HitRegister(pub [Option<HitInfo>; 4]);

#[derive(Deref, DerefMut)]
pub struct SongTime(pub R32);

#[derive(Deref, DerefMut)]
pub struct AutomationOutputTable<T>(pub [ChannelOutput<T>; CHANNELS_PER_AUTOMATION_TABLE]);
