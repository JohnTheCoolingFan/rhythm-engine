use crate::{automation::*, utils::*};
use crate::utils::seeker::*;
use super::*;

pub enum Lead {
    Into,
    OutOf,
}

struct ResponseConfig {
    delay: f32,

}

pub enum Response {
    Ignore,
    Toggle(bool),
    Commence(bool),
    Halt(bool),
    Jump{
        offset: f32
    },
    InverseReset{
        offset: f32
    },
    Follow{
        halt_delay: f32
    }
}

impl<T> SignalResponse<T> {
    pub fn respond(&mut self) {
        match self {
            Self::Toggle(_, ref mut b) => *b = !*b,
            Self::Commence(_, ref mut b) => *b = true,
            Self::Halt(_, ref mut b) => *b = false,
            _ => {}
        }
    }

    pub fn unwrap(&self) -> &T {
        match self {
            Self::Ignore(ref val) 
            | Self::Toggle(ref val, _)
            | Self::Commence(ref val, _)
            | Self::Halt(ref val, _) => val
        }
    }

    pub fn unwrap_mut(&mut self) -> &mut T {
        match self {
            Self::Ignore(ref mut val) 
            | Self::Toggle(ref mut val, _)
            | Self::Commence(ref mut val, _)
            | Self::Halt(ref mut val, _) => val
        }
    }

}

pub type Channel<T> = Vec<Epoch<T>>;

pub struct PlayList<T> {
    channels: Vec<Channel<SignalResponse<T>>>,
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

/*type BPMSeeker<'a> = BPSeeker<'a, Epoch<BPM>>;
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
        else {
            curr.val
        }
    }
}*/
//
//
//
//
//
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
