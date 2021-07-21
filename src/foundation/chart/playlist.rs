use crate::utils::{SConst, Seekable};
use std::{marker::PhantomData, ops::Index};

pub type Channel<'a, T: Seekable<'a>> = Vec<(f32, T)>;

struct PlayList<'a, T> {
    channels: Vec<Channel<'a, T>>,
    _phantom: PhantomData<&'a T>,
}

struct Statics<'a> {
    sense_muls: Channel<'a, SConst<f32>>,
    _phantom: PhantomData<&'a ()>,
}
