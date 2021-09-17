use crate::{automation::*, utils::*};
use duplicate::duplicate;
use std::ptr::eq;
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

struct HitInfo {
    //  the time the object is supposed to be hit instead of when it actually is hit
    //  this way animations will always be in sync with the music
    //  reguardless of how accurate the hit was
    obj_time: f32,
    layer: u8
}

struct SignalResponse<'a, T>
where
    T: Seekable<'a>
{
    response: Response,
    layer: u8,
    target: T
}

impl<'a, T> SignalResponse<'a, T>
where
    T: Seekable<'a>
{
    //  Holds will behave like hits for implementation simplicity
    //  And because I can't think of scenarios where Hold behavior
    //  would be useful. Might change in future tho.
    pub fn respond(&mut self, hits: &[Option<HitInfo>; 4]) {
        for hit in hits.iter().flatten() {
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
//
//
//
//
pub type Channel<T> = Vec<Epoch<T>>;
pub type PlayList<'a, T> = Vec<Channel<SignalResponse<'a, T>>>;
pub type PlayListSeeker<'a, T> = Seeker<
    (), 
    Vec<(
        Seeker<&'a Channel<SignalResponse<'a, T>>, usize>,
        <T as Seekable<'a>>::Seeker
    )>
>;

impl<'a, T> SeekerTypes for PlayListSeeker<'a, T>
where
    T: Seekable<'a>
{
    type Source = Epoch<SignalResponse<'a, T>>;
    type Output = Vec<<<T as Seekable<'a>>::Seeker as SeekerTypes>::Output>;
}

impl<'a, T> Seek for PlayListSeeker<'a, T>
where
    T: Seekable<'a>
{
    #[duplicate(method; [jump]; [seek])]
    fn method(&mut self, offset: f32) -> Self::Output {
        /*let initial_pass: Self::Output = */
        self.meta
            .iter_mut()
            .map(|(channel_seeker, item_seeker)| {
                item_seeker = if !eq(&channel_seeker.method(offset).val, item_seeker.data) {
                }
            })
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

pub struct Globals<'a> {
    sense_muls: Channel<f32>,
    bpms: Vec<Epoch<BPM>>,
    camera_pos: Channel<SignalResponse<'a, ComplexSpline>>,
    camera_rot: Channel<SignalResponse<'a, TransformPoint<Rotation>>>,
    camera_scale: Channel<SignalResponse<'a, TransformPoint<Scale>>>
}

pub struct LiveChart<'a> {
    poly_entities: Vec<PolyEntity>,
    rotations: PlayList<'a, TransformPoint<Rotation>>,
    scale: PlayList<'a, TransformPoint<Scale>>,
    splines: PlayList<'a, ComplexSpline>,
    colours: PlayList<'a, DynColor>
}
