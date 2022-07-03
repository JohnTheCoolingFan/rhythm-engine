use std::hint::unreachable_unchecked;

use bevy::prelude::*;
use derive_more::{Deref, DerefMut, From};
use itertools::Itertools;
use lyon_geom::*;
use noisy_float::prelude::*;

use crate::{
    automation::{Channel, ChannelSeeker},
    bounds::*,
    resources::*,
    utils::*,
};

#[derive(Clone, Copy)]
pub enum Curvature {
    Linear,
    Circular(Vec2),
    Quadratic(Vec2),
    Cubic(Vec2, Vec2),
}

pub struct Segment {
    curvature: Curvature,
    end: Vec2,
}

pub enum SampledCurve {
    Points(Vec<(Vec2, R32)>),
    CircleArc {
        start: Vec2,
        center: Vec2,
        theta: R32,
    },
}

pub struct SampledSegment {
    path_offset: R32,
    sample: SampledCurve,
}

impl Quantify for SampledSegment {
    fn quantify(&self) -> R32 {
        self.path_offset
    }
}

#[derive(Component, Deref, DerefMut, From)]
pub struct Spline(Vec<Segment>);

#[derive(Component, Deref, DerefMut, From)]
pub struct SplineLut(pub Vec<SampledSegment>);

impl ControllerTable for SplineLut {
    type Item = SampledSegment;
    fn table(&self) -> &[Self::Item] {
        self.as_slice()
    }
}

impl Spline {
    fn create_line(spline_length: &mut R32, start: Vec2, end: Vec2) -> SampledCurve {
        let segment_length = r32(start.distance(end));
        *spline_length += segment_length;
        SampledCurve::Points(vec![(start, r32(0.)), (end, segment_length)])
    }

    #[rustfmt::skip]
    fn create_bezier(
        spline_length: &mut R32,
        start: Vec2,
        points: impl Iterator<Item = Point<f32>>
    )
        -> SampledCurve
    {
        let tail = [start]
            .into_iter()
            .chain(points.map(|p| Vec2::new(p.x, p.y)))
            .tuple_windows::<(_, _)>()
            .scan(r32(0.), |segment_length, (prev, current)| {
                *segment_length += r32(prev.distance(current));
                Some((current, *segment_length))
            });

        let sampled = [(start, r32(0.))]
            .into_iter()
            .chain(tail)
            .collect::<Vec<_>>();

        *spline_length += sampled.last().unwrap().1;
        SampledCurve::Points(sampled)
    }

    fn create_arc(spline_length: &mut R32, start: Vec2, ctrl: Vec2, end: Vec2) -> SampledCurve {
        //https://math.stackexchange.com/a/1460096
        let m11_determinant = [start, ctrl, end]
            .map(|point| [point.x, point.y, 1.])
            .into_matrix()
            .determinant();

        if m11_determinant.abs() <= f32::EPSILON {
            Self::create_line(spline_length, start, end)
        } else {
            let m12 = [start, ctrl, end]
                .map(|point| [point.x.powi(2) + point.y.powi(2), point.y, 1.])
                .into_matrix();

            let m13 = [start, ctrl, end]
                .map(|point| [point.x.powi(2) + point.y.powi(2), point.x, 1.])
                .into_matrix();

            let center = Vec2::new(
                0.5 * (m12.determinant() / m11_determinant),
                -0.5 * (m13.determinant() / m11_determinant),
            );

            let (a, b) = (center - start, center - end);
            let theta = (a.dot(b) / (a.length() * b.length())).acos().to_degrees();

            SampledCurve::CircleArc {
                center,
                start,
                theta: r32(
                    match (
                        [start, ctrl, end].into_iter().orientation(),
                        [start, center, end].into_iter().orientation(),
                    ) {
                        (ctrl_o, center_o) if ctrl_o != center_o => theta,
                        _ => theta.signum() * (360. - theta.abs()),
                    },
                ),
            }
        }
    }

    #[rustfmt::skip]
    fn sample_segment(
        spline_length: &mut R32,
        (&Segment { end: start, .. }, &Segment { curvature, end }): (&Segment, &Segment),
    )
        -> Option<SampledSegment>
    {
        Some(SampledSegment {
            path_offset: *spline_length,
            sample: match curvature {
                Curvature::Linear => Self::create_line(spline_length, start, end),
                Curvature::Circular(ctrl) => Self::create_arc(spline_length, start, ctrl, end),
                Curvature::Quadratic(ctrl) => {
                    let bezier = QuadraticBezierSegment {
                        from: start.to_array().into(),
                        ctrl: ctrl.to_array().into(),
                        to: end.to_array().into(),
                    };

                    Self::create_bezier(spline_length, start, bezier.flattened(0.05))
                }
                Curvature::Cubic(a, b) => {
                    let bezier = CubicBezierSegment {
                        from: start.to_array().into(),
                        ctrl1: a.to_array().into(),
                        ctrl2: b.to_array().into(),
                        to: end.to_array().into(),
                    };

                    Self::create_bezier(spline_length, start, bezier.flattened(0.05))
                }
            },
        })
    }

    #[rustfmt::skip]
    pub fn sample(&self) -> SplineLut {
        [Segment { curvature: Curvature::Linear, end: Vec2::new(0., 0.) }]
            .iter()
            .chain(self.iter())
            .tuple_windows::<(_, _)>()
            .scan(r32(0.), Self::sample_segment)
            .collect::<Vec<_>>()
            .into()
    }
}

#[derive(Component)]
struct SplineLutIndexCache {
    segment: usize,
    path: usize,
}

// Reuse automation system
// Use only 1 bound array

fn eval_splines(
    spline_luts: Query<(&Channel<Displacement, (Spline, SplineLut)>, &ChannelSeeker)>,
    displacements: Res<AutomationOutputTable<Displacement>>,
    mut points: ResMut<AutomationOutputTable<Vec2>>,
) {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;
}
