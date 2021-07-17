use crate::foundation::*;
use crate::utils::*;

pub struct Chart<'a> {
    poly_entities: PolyEntity,
    rotations: Playlist<'a, TransformPoint<Rotation>>,
    scale: Playlist<'a, TransformPoint<Scale>>,
    splines: Playlist<'a, ComplexSpline>,
    audio: String,
}
