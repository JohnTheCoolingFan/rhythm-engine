use std::hint::unreachable_unchecked;

use bevy::prelude::*;
use derive_more::{Deref, DerefMut, From};
use itertools::Itertools;
use lyon_geom::*;
use noisy_float::prelude::*;

use crate::{resources::*, utils::*};

enum Curvature {
    Linear,
    Circular(Vec2),
    Quadratic(Vec2),
    Cubic(Vec2, Vec2),
}

struct Segment {
    curvature: Curvature,
    end: Vec2,
}

pub enum SampledCurve {
    Points(Vec<(Vec2, R32)>),
    Arc {
        start: Vec2,
        center: Vec2,
        theta: Option<R32>,
    },
}

pub struct SampledSegment {
    path_displacement: R32,
    sample: SampledCurve,
}

impl Quantify for SampledSegment {
    fn quantify(&self) -> R32 {
        self.path_displacement
    }
}

#[derive(Component, Deref, DerefMut, From)]
struct Spline(Vec<Segment>);

#[derive(Component, Deref, DerefMut, From)]
pub struct SplineLut(pub Vec<SampledSegment>);

impl Spline {
    fn sample_bezier(start: Vec2, flattened: impl Iterator<Item = Point<f32>>) -> Vec<(Vec2, R32)> {
        let tail = [start]
            .into_iter()
            .chain(flattened.map(|p| Vec2::new(p.x, p.y)))
            .tuple_windows::<(_, _)>()
            .scan(r32(0.), |segment_length, (prev, current)| {
                *segment_length += r32(prev.distance(current));
                Some((current, *segment_length))
            });

        [(start, r32(0.))].into_iter().chain(tail).collect()
    }

    #[rustfmt::skip]
    fn sample(&self) -> SplineLut {
        [Segment { curvature: Curvature::Linear, end: Vec2::new(0., 0.) }]
            .iter()
            .chain(self.iter())
            .tuple_windows::<(_, _)>()
            .scan(r32(0.), |path_length, (Segment { end: start, .. }, Segment { curvature, end })| {
                Some(SampledSegment {
                    path_displacement: *path_length,
                    sample: match curvature {
                        Curvature::Linear => {
                            let segment_length = r32(start.distance(*end));
                            *path_length += segment_length;

                            SampledCurve::Points(vec![
                                (*start, r32(0.)),
                                (*end, segment_length)
                            ])
                        }
                        Curvature::Circular(ctrl) => { //https://math.stackexchange.com/a/1460096
                            let m11_determinant = [start, ctrl, end]
                                .map(|point| [point.x, point.y, 1.])
                                .into_matrix()
                                .determinant();

                            if f32::EPSILON < m11_determinant.abs() {
                                let m12 = [start, ctrl, end]
                                    .map(|point| [point.x.powi(2) + point.y.powi(2), point.y, 1.])
                                    .into_matrix();

                                let m13 = [start, ctrl, end]
                                    .map(|point| [point.x.powi(2) + point.y.powi(2), point.x, 1.])
                                    .into_matrix();

                                let center = Vec2::new(
                                    0.5 * (m12.determinant() / m11_determinant),
                                    -0.5 * (m13.determinant() / m11_determinant)
                                );

                                let (center_to_start, center_to_end) = (
                                    *start - center, *end - center
                                );
                                unimplemented!();
                            }
                            else {
                                let segment_length = r32(start.distance(*end));
                                *path_length += segment_length;

                                SampledCurve::Points(vec![
                                    (*start, r32(0.)),
                                    (*end, segment_length)
                                ])
                            }
                        }
                        Curvature::Quadratic(ctrl) => {
                            let bezier = QuadraticBezierSegment {
                                from: start.to_array().into(),
                                ctrl: ctrl.to_array().into(),
                                to: end.to_array().into()
                            };

                            let sampled = Self::sample_bezier(*start, bezier.flattened(0.05));
                            *path_length += sampled.last().unwrap().1;
                            SampledCurve::Points(sampled)
                        }
                        Curvature::Cubic(a, b) => {
                            let bezier = CubicBezierSegment {
                                from: start.to_array().into(),
                                ctrl1: a.to_array().into(),
                                ctrl2: b.to_array().into(),
                                to: end.to_array().into()
                            };

                            let sampled = Self::sample_bezier(*start, bezier.flattened(0.05));
                            *path_length += sampled.last().unwrap().1;
                            SampledCurve::Points(sampled)
                        }
                    }
                })
            })
            .collect::<Vec<_>>()
            .into()
    }
}

#[derive(Component)]
struct SplineIndexCache {
    segment: usize,
    path: Option<usize>,
}

fn sync_spline_luts(splines: Query<Changed<Spline>>, mut luts: ResMut<SplineLuts>) {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn geom() {
        let bezier = QuadraticBezierSegment::<f32> {
            from: Point::new(0.0, 0.0),
            ctrl: Point::new(0.0, 1.0),
            to: Point::new(1.0, 1.0),
        };

        println!("{:#?}", bezier.flattened(0.05).collect::<Vec<_>>())
    }
}
