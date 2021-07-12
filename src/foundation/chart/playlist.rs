use std::marker::PhantomData;
use crate::utils::Seekable;

pub struct Playlist<'a, T> 
    where T: Seekable<'a>
{
    rotations: Vec<Vec<(f32, T)>>,
    _phantom: PhantomData<&'a T>
}

struct PLSeeker<'a, T> 
    where T: Seekable<'a>,
{
    outputs: Vec<T::Output>,
    seekers: Vec<T::SeekerType>
}

impl<'a, T> Playlist<'a, T> 
    where T: Seekable<'a>
{
    pub fn move_track(&mut self, channel_id: usize, track_id: usize, y: usize, x: f32) {

    }
}
