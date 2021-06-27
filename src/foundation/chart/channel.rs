use std::marker::PhantomData;

use crate::utils::Seekable;

struct Channel<'a, T>
    where T: Seekable<'a>
{
    offsets: Vec<f32>,
    tracks: Vec<T>,
    _pd: PhantomData<&'a T>
}

impl<'a, T: Seekable<'a>> Channel<'a, T> {
}

//impl Seekable for Channel<>
