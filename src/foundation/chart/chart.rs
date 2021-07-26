use crate::foundation::*;
use crate::utils::*;

pub struct Chart<'a> {
    poly_entities: PolyEntity,
    rotations: PlayList<'a, TransformPoint<Rotation>>,
    scale: PlayList<'a, TransformPoint<Scale>>,
    splines: PlayList<'a, ComplexSpline>,
    audio: String,
}
