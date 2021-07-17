use crate::utils::Seekable;
use std::{marker::PhantomData, ops::Index};

pub struct Channel<'a, T>
where
    T: Seekable<'a>,
{
    tracks: Vec<(f32, T)>,
    _phantom: PhantomData<&'a T>,
}

pub struct Playlist<'a, T>
where
    T: Seekable<'a>,
{
    channels: Vec<Channel<'a, T>>,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T> Playlist<'a, T>
where
    T: Seekable<'a>,
{
    pub fn move_track(&mut self, channel_id: usize, track_id: usize, y: usize, x: f32) {}
}
