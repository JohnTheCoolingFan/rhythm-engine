use crate::{automation::*, utils::*};
use duplicate::duplicate;
use std::ops::Index;
use super::*;

pub enum Response {
    Ignore,
    Commence{
        started: bool
    },
    Switch {
        delegate: usize,
        switched: bool
    },
    Toggle {
        delegate: usize,
        switched: bool
    },
    Follow {
        excess: f32,
        last_hit: Option<f32>,
    }
}

pub struct HitInfo {
    //  
    //  [CLARIFICATION]
    //  
    //  the time the object is supposed to be hit instead of when it actually is hit
    //  this way animations will always be in sync with the music
    //  reguardless of how accurate the hit was
    obj_time: f32,
    layer: u8
}

pub struct SignalResponse<T> {
    response: Response,
    layer: u8,
    target: T
}

impl<'a, T> SignalResponse<T>
where
    T: Seekable<'a>
{
    //  
    //  [CLARIFICATION]
    //  
    //  Holds will behave like hits for implementation simplicity
    //  And because I can't think of scenarios where Hold behavior
    //  would be useful. Might change in future tho.
    pub fn respond(&mut self, hits: &[Option<HitInfo>; 4]) {
        for hit in hits.iter().flatten() {
            if hit.layer == self.layer {
                match self.response {
                    Response::Commence{ ref mut started } => *started = true,
                    Response::Switch{ ref mut switched, .. } => *switched = true,
                    Response::Toggle{ ref mut switched, .. } => *switched = !*switched,
                    Response::Follow{ ref mut last_hit, .. } => *last_hit = Some(hit.obj_time),
                    _ => {}
                }
            }
        }
    }

    //
    //  [MISSING IMPL]
    //
    //pub fn translate(&self, t: f32) -> f32 {
    //}
}
//
//
//
//
//
pub type Channel<T> = Vec<Epoch<SignalResponse<T>>>;
pub type ChannelSeeker<'a, T> = Seeker<(), (
    Seeker<&'a [Epoch<SignalResponse<T>>], usize>, 
    <T as Seekable<'a>>::Seeker
)>;

impl<'a, T> SeekerTypes for ChannelSeeker<'a, T>
where
    T: Seekable<'a>
{
    type Source = Epoch<SignalResponse<T>>;
    type Output = <<T as Seekable<'a>>::Seeker as SeekerTypes>::Output;
}

impl<'a, T> Seek for ChannelSeeker<'a, T> 
where
    T: Seekable<'a>,
    <T as Seekable<'a>>::Seeker: SeekerTypes<Source = Self::Source>,
    Seeker<&'a [Epoch<SignalResponse<T>>], usize>: SeekerTypes<Source = Self::Source> + Exhibit + Seek
{
    #[duplicate(method; [seek]; [jump])]
    fn method(&mut self, offset: f32) -> Self::Output {
        let Seeker{ meta: (outer, inner), ..} = self;
        let old = outer.meta;
        outer.method(offset); 
        match outer.meta { //need to manually index cause lifetimes
            oob if outer.data.len() <= oob => {
                outer.data[FromEnd(0)].val.target.seeker().jump(
                    offset - outer.data[FromEnd(0)].offset
                )
            },
            index => {
                if index != old {
                    *inner = outer.data[index].val.target.seeker();
                }
                inner.method(offset - outer.data[index].offset)
            }
        }
    }
}

pub type PlayList<T> = Vec<Channel<T>>;
pub type PlayListSeeker<'a, T> = Seeker<(),Vec<(
    ChannelSeeker<'a, T>,
    <<T as Seekable<'a>>::Seeker as SeekerTypes>::Output
)>>;

impl<'a, T> SeekerTypes for PlayListSeeker<'a, T>
where
    T: Seekable<'a>,
{
    type Source = Epoch<SignalResponse<T>>;
    type Output = ();
}

impl<'a, T> Seek for PlayListSeeker<'a, T>
where
    T: Seekable<'a>,
    <T as Seekable<'a>>::Seeker: SeekerTypes<Source = Self::Source>,
    ChannelSeeker<'a, T>:  Exhibit + Seek + SeekerTypes<
        Source = Self::Source,
        Output = <<T as Seekable<'a>>::Seeker as SeekerTypes>::Output
    >
{

    #[duplicate(method; [seek]; [jump])]
    fn method(&mut self, offset: f32) {
        self.meta
            .iter_mut()
            .for_each(|(seeker, output)| *output = seeker.method(offset));

        //  
        //  [MISSING IMPL]
        //
        //  Add output delegation
    }
}

impl<'a, T> Index<usize> for PlayListSeeker<'a, T>
where
    T: Seekable<'a>
{
    type Output = <<T as Seekable<'a>>::Seeker as SeekerTypes>::Output;

    fn index(&self, index: usize) -> &Self::Output {
        &self.meta[index].1
    }
}
//
//
//
//
//
#[derive(Clone, Copy)]
pub struct Bpm {
    bpm: f32,
    signature: i32
}

impl Default for Bpm {
    fn default() -> Self {
        Self {
            bpm: 120.,
            signature: 4
        }
    }
}

type BpmSeeker<'a> = Seeker<&'a [Epoch<Bpm>], usize>;

impl<'a> Exhibit for BpmSeeker<'a> {
    fn exhibit(&self, _: f32) -> Bpm {
        match self.current() {
            Ok(bpm) | Err(bpm) => bpm.val
        }
    }
}
//
//
//
//
//
struct SongMetaData<T> {
    pub artists: String,
    pub title: String,
    pub authors: TVec<String>,
    //used in editor
    pub extra: T, //Vec<Epoch<Bpm>>
}

pub struct Chart<T> {
    pub audio_source: String,
    //globals
    pub sense_muls: Vec<Epoch<f32>>,
    pub camera_pos: usize,
    pub camera_rot: usize,
    pub camera_scale: usize,
    //live data
    pub poly_entities: Vec<PolyEntity>,
    pub rotations: PlayList<TransformPoint<Rotation>>,
    pub scale: PlayList<TransformPoint<Scale>>,
    pub splines: PlayList<ComplexSpline>,
    pub colours: PlayList<DynColor>,
    //meta data: only deserialized in editor and menu
    pub meta: T,
}
