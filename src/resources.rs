use bevy::prelude::*;
use derive_more::From;

use crate::{hit::HitInfo, utils::*};

#[derive(Deref, DerefMut, From)]
pub struct HitRegister(pub [Option<HitInfo>; 4]);

#[derive(Clone, Copy, Deref, DerefMut, From)]
pub struct SongTime(pub P32);
