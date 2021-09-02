use crate::{automation::*, utils::*};
use crate::utils::seeker::*;
use super::*;

pub enum SignalResponse<T> {
    Ignore(T),
    Toggle(T, bool),
    Commence(T, bool),
    Halt(T, bool),
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
enum BPM {
    Static{ bpm: f32 },
    Variable{ division_factor: i32 },
}

type BPMSeeker<'a> = BPSeeker<'a, BPM>;
/*impl<'a> Exhibit for BPMSeeker<'a> {
    fn exhibit(&self, offset: f32) -> () {
        let (curr, prev) = (self.current(), self.previous());
        match prev.val {
            Interpret::Individual()
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
