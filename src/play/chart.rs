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

struct SignalResponse<T> {
    response: Response,
    target: T
}

impl<T> SignalResponse<T> {
    pub fn respond(&mut self, hit_time: f32) {
        match self.response {
            Response::Commence{ ref mut started } => *started = true,
            Response::Switch{ ref mut switched, .. } => *switched = true,
            Response::Toggle{ ref mut switched, .. } => *switched = !*switched,
            Response::Follow{ ref mut last_hit, .. } => *last_hit = Some(hit_time),
            _ => {}
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
