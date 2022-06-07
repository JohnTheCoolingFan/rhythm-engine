use noisy_float::prelude::*;

use crate::hit::*;

pub const CHANNELS_PER_TABLE: usize = 256;

pub struct HitRegister(pub [Option<HitInfo>; 4]);

pub struct SongTime(pub R32);

pub struct ChannelOutput<T> {
    pub output: Option<T>,
    pub redirect: Option<usize>,
}

pub struct OutputTable<T>(pub [ChannelOutput<T>; CHANNELS_PER_TABLE]);
