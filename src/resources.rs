use bevy::prelude::*;
use derive_more::From;
use noisy_float::prelude::*;

use crate::hit::HitInfo;

#[derive(Deref, DerefMut, From)]
pub struct HitRegister(pub [Option<HitInfo>; 4]);

#[derive(Clone, Copy, Deref, DerefMut, From)]
pub struct SongTime(pub R32);
