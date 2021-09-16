use crate::{automation::*, utils::*};
use crate::utils::seeker::*;
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

struct SignalResponse<T> {
    response: Response,
    layer: u8,
    target: T
}

impl<T> SignalResponse<T> {
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

pub struct PlayList<T> {
    channels: Vec<Channel<SignalResponse<T>>>,
}

impl<'a, T> PlayList<T>
where
    T: Seekable<'a>
{
    fn make_table(&self, t: f32, hits: &[Option<HitInfo>; 4]) {
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

impl<'a> Exhibit for BPMSeeker<'a> {
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
}
//
//
//
//
//
enum AudioSourceType {
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
    pub audio: String,
    //hash or song ID? 
}

pub struct Globals {
    sense_muls: Channel<f32>,
    bpms: Vec<Epoch<BPM>>,
    camera_pos: Channel<SignalResponse<ComplexSpline>>,
    camera_rot: Channel<SignalResponse<Rotation>>,
    camera_scale: Channel<SignalResponse<Scale>>
}

pub struct LiveChart {
    poly_entities: Vec<PolyEntity>,
    rotations: PlayList<TransformPoint<Rotation>>,
    scale: PlayList<TransformPoint<Scale>>,
    splines: PlayList<ComplexSpline>,
    colours: PlayList<DynColor>
}
