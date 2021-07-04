use crate::foundation::{
    automation::{
        dyn_color::DynColor,
        point_transform::PointTransform
    },
    complex_spline::ComplexSpline,
    poly_entity::*
};

struct Channel<T>
{
    tracks: Vec<(f32, T)>,    //VecDeque + Thread? Bounded Channel? Check memory usage firs
}

impl Channel <PointTransform> {
}

impl Channel <DynColor> {
}

impl Channel <ComplexSpline> {
}

impl Channel <PolyEntity> {
}

8 //need to possibly change seeker trait
