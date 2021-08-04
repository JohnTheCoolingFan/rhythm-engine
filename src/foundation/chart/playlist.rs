use crate::utils::seeker::*;
use std::marker::PhantomData;

pub type Channel<'a, T> = Vec<SimpleAnchor<T>>;

pub struct PlayList<'a, T> {
    channels: Vec<Channel<'a, T>>,
    _phantom: PhantomData<&'a T>,
}

pub struct Statics<'a> {
    sense_muls: Channel<'a, f32>,
    _phantom: PhantomData<&'a ()>,
}
