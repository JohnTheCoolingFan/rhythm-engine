use crate::foundation::*;

pub struct Chart<'a> {
    poly_entities: PolyEntity,
    rotations: Playlist<'a, Automation>,
    scale: Playlist<'a, Automation>,
    splines: Playlist<'a, ComplexSpline>,
    audio: String,
}
