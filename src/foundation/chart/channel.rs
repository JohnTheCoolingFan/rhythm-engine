use std::marker::PhantomData;
use crate::utils::Seekable;

struct Channel<'a, T>
    where T: Seekable<'a>
{
    tracks: Vec<(f32, T)>,    //VecDeque + Thread? Bounded Channel? Check memory usage firs
    _phantom: PhantomData<&'a T>
}

struct Playlist {
}
