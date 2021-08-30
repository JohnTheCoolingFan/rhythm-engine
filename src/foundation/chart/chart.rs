use crate::foundation::*;

struct SongMetaData {
    pub artists: String,
    pub title: String,
    pub audio: String
}

pub struct Chart {
    song_meta: SongMetaData,
    poly_entities: Vec<PolyEntity>,
    rotations: PlayList<TransformPoint<Rotation>>,
    scale: PlayList<TransformPoint<Scale>>,
    splines: PlayList<ComplexSpline>,
    colours: PlayList<DynColor>
}
