use crate::{foundation::*, utils::*};

struct SongMetaData {
    pub artists: String,
    pub title: String,
    pub audio: String,
    //hash or song ID? 
}

type BPMMarker = Epoch<Interpret<(f32, f32)>>;
type BPMSeeker<'a> = BPSeeker<'a, BPMMarker>;
impl<'a> Exhibit for BPMSeeker<'a> {
    fn exhibit(&self, offset: f32) -> {
    }
}


pub struct Globals {
    sense_muls: Channel<f32>,
    bpms: Vec<BPMMarker>,
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
