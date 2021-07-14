use crate::utils::Seekable;
use std::{marker::PhantomData, ops::Index};

pub struct Playlist<'a, T>
where
    T: Seekable<'a>,
{
    rotations: Vec<Vec<(f32, T)>>,
    _phantom: PhantomData<&'a T>,
}

pub struct PLSeeker<'a, T>
where
    T: Seekable<'a>,
{
    outputs: Vec<T::Output>,
    seekers: Vec<T::SeekerType>,
}

impl<'a, T> Index<usize> for PLSeeker<'a, T>
where
    T: Seekable<'a>,
{
    type Output = T::Output;
    fn index(&self, index: usize) -> &Self::Output {
        &self.outputs[index]
    }
}

impl<'a, T> Playlist<'a, T>
where
    T: Seekable<'a>,
{
    pub fn move_track(&mut self, channel_id: usize, track_id: usize, y: usize, x: f32) {}
}
