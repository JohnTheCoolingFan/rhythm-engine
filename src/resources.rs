use noisy_float::prelude::*;

use crate::hit::*;

pub const CHANNELS_PER_TABLE: usize = 256;

struct HitRegister([HitInfo; 4]);

pub struct SongTime(pub N32);

pub struct ChannelOutput<T> {
    output: Option<T>,
    redirect: Option<usize>,
}

pub struct OutputTable<T>(pub [T; CHANNELS_PER_TABLE]);
