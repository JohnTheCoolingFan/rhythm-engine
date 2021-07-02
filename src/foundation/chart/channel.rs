use std::marker::PhantomData;

use crate::utils::Seekable;

struct Channel<'a, T>
    where T: Seekable<'a>
{
    offsets: Vec<f32>,  //VecDeque + Thread? Bounded Channel?
    tracks: Vec<T>,     //Check memory usage first
    _pd: PhantomData<&'a T>
}

impl<'a, T: Seekable<'a>> Channel<'a, T> {
}

