use crate::utils::{misc::*, seeker::*};
use std::{marker::PhantomData, ops::Index};

pub type Channel<'a, T: Seekable<'a>> = Vec<(f32, T)>;

pub struct PlayList<'a, T> {
    channels: Vec<Channel<'a, T>>,
    _phantom: PhantomData<&'a T>,
}

pub struct Statics<'a> {
    sense_muls: Channel<'a, SeekableQuantum<f32>>,
    _phantom: PhantomData<&'a ()>,
}
