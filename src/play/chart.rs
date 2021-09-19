use crate::{automation::*, utils::*};
use duplicate::duplicate;
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
    //  the time the object is supposed to be hit instead of when it actually is hit
    //  this way animations will always be in sync with the music
    //  reguardless of how accurate the hit was
    obj_time: f32,
    layer: u8
}

pub struct SignalResponse<T>
{
    response: Response,
    layer: u8,
    target: T
}

impl<'a, T> SignalResponse<T>
where
    T: Seekable<'a>
{
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
                    *inner = outer.data[FromEnd(0)].val.target.seeker();
                }
                inner.method(outer.data[index].offset)
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
    T: Seekable<'a>
{
    type Source = Epoch<SignalResponse<T>>;
    type Output = Vec<<<T as Seekable<'a>>::Seeker as SeekerTypes>::Output>;
}

impl<'a, T> Seek for PlayListSeeker<'a, T>
where
    T: Seekable<'a>
{

    #[duplicate(method; [seek]; [jump])]
    fn method(&mut self, offset: f32) -> Self::Output {
        self.meta
            .iter_mut()
            .map(|(seeker, output)| output = seeker.method())
            .collect()
    }
}
//
//
//
//
//
#[derive(Clone, Copy)]
pub enum BPM {
    Defined {
        bpm: f32,
        signature: i32
    },
    Undefined {
        division_factor: f32,
        signature: i32
    },
}

impl Default for BPM {
    fn default() -> Self {
        Self::Undefined {
            division_factor: 1.,
            signature: 1
        }
    }
}

type BPMSeeker<'a> = Seeker<&'a TVec<Epoch<BPM>>, usize>;

/*impl<'a> Exhibit for BPMSeeker<'a> {
    fn exhibit(&self, offset: f32) -> BPM {
        match (self.previous(), self.current()) {
            match prev.val {
                BPM::Defined{ .. } => prev.val,
                BPM::Undefined{ division_factor, signature } => {
                    let interval = curr.time - prev.time;
                    BPM::Defined{
                        bpm: (60. * 1000.) / (interval / division_factor),
                        signature
                    }
                }
            }
        }
    }
}*/
//
//
//
//
//
enum AudioSource {
    File(String),
    //For later
    Stream {
        url: String,
        service: String
    }
}

struct SongMetaData {
    pub artists: String,
    pub title: String,
    pub audio: AudioSource,
    //hash or song ID? 
}

pub struct Globals {
    sense_muls: Channel<f32>,
    bpms: Vec<Epoch<BPM>>,
    camera_pos: Channel<ComplexSpline>,
    camera_rot: Channel<TransformPoint<Rotation>>,
    camera_scale: Channel<TransformPoint<Scale>>
}

pub struct LiveChart {
    poly_entities: Vec<PolyEntity>,
    rotations: PlayList<TransformPoint<Rotation>>,
    scale: PlayList<TransformPoint<Scale>>,
    splines: PlayList<ComplexSpline>,
    colours: PlayList<DynColor>
}
