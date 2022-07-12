use core::iter::once as iter_once;
use std::f32::consts::PI;

use bevy::prelude::*;
use itertools::Itertools;
use lyon_geom::*;
use noisy_float::prelude::*;

use crate::{automation::*, utils::*};

pub enum Sample {
    Point {
        displacement: R32,
        position: Vec2,
    },
    Arc {
        displacement: R32,
        center: Vec2,
    },
}

impl Quantify for Sample {
    fn quantify(&self) -> R32 {
        match self {
            Self::Point { displacement, .. } | Self::Arc { displacement, .. } => *displacement,
        }
    }
}

impl Lerp for Sample {
    type Output = Vec2;
    #[rustfmt::skip]
    fn lerp(&self, next: &Self, t: T32) -> Self::Output {
        match (self, next) {
            (Sample::Point { position: start, .. }, Sample::Point { position: end, .. }) => {
                start.lerp(*end, t.raw())
            },
            (Sample::Point { position: start, .. }, Sample::Arc { center, displacement }) => {
                center.rotate(start, (*displacement / center.distance(*start)).to_degrees())
            }
            _ => unreachable!()
        }
    }
}

#[derive(Clone, Copy)]
pub enum Curvature {
    Linear,
    Circular(Vec2),
    Quadratic(Vec2),
    Cubic(Vec2, Vec2),
}

pub struct Segment {
    curvature: Curvature,
    position: Vec2,
}

#[rustfmt::skip]
impl Segment {
    fn sample_bezier(
        path_length: &mut R32,
        start: Vec2,
        points: impl Iterator<Item = Point<f32>>
    )
        -> Vec<Sample>
    {
        [start]
            .into_iter()
            .chain(points.map(|p| Vec2::new(p.x, p.y)))
            .tuple_windows::<(_, _)>()
            .map(|(prev, curr)| {
                *path_length += prev.distance(curr);
                Sample::Point {
                    position: curr,
                    displacement: *path_length
                }
            })
            .collect::<Vec<_>>()
    }

    fn sample(&self, path_length: &mut R32, start: Vec2) -> Vec<Sample> {
        match self.curvature {
            Curvature::Linear => {
                *path_length += start.distance(self.position);
                vec![Sample::Point {
                    position: self.position,
                    displacement: *path_length
                }]
            },
            Curvature::Circular(ctrl) => {
                let end = self.position;
                //https://math.stackexchange.com/a/1460096
                let m11_determinant = [start, ctrl, end]
                    .map(|point| [point.x, point.y, 1.])
                    .into_matrix()
                    .determinant();

                if m11_determinant.abs() <= f32::EPSILON {
                    *path_length += start.distance(self.position);
                    vec![Sample::Point {
                        position: end,
                        displacement: *path_length
                    }]
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

                    let theta = match (
                        [start, ctrl, end].into_iter().orientation(),
                        [start, center, end].into_iter().orientation(),
                        (a.dot(b) / (a.length() * b.length())).acos().to_degrees()
                    ) {
                        (ctrl_o, center_o, theta) if ctrl_o != center_o => theta,
                        (_, _, theta) => theta.signum() * (360. - theta.abs()),
                    };

                    let displacement = r32(
                        2. * PI * ((theta * center.distance(start)).abs() / 360.)
                    );

                    let samples = [
                        (f32::EPSILON < center.distance(start)).then(|| Sample::Arc {
                            center,
                            displacement,
                        }),
                        Some(Sample::Point {
                            position: end,
                            displacement
                        })
                    ];

                    samples.into_iter().flatten().collect()
                }
            }
            Curvature::Quadratic(ctrl) => {
                let quadratic = QuadraticBezierSegment {
                    from: start.to_array().into(),
                    ctrl: ctrl.to_array().into(),
                    to: self.position.to_array().into(),
                };

                Self::sample_bezier(
                    path_length,
                    start,
                    quadratic.flattened(0.05)
                )
            }
            Curvature::Cubic(a, b) => {
                let cubic = CubicBezierSegment {
                    from: start.to_array().into(),
                    ctrl1: a.to_array().into(),
                    ctrl2: b.to_array().into(),
                    to: self.position.to_array().into(),
                };

                Self::sample_bezier(
                    path_length,
                    start,
                    cubic.flattened(0.05)
                )
            }
        }
    }
}

pub struct Waypoint {
    displacement: R32,
    weight: Weight,
}

struct Repeater;

#[derive(Component)]
pub struct SmartSpline {
    pub path: Vec<Segment>,
    pub lut: Vec<Sample>,
    pub waypoints: Vec<Waypoint>,
    repeater: Option<Repeater>,
}

impl SmartSpline {
    #[rustfmt::skip]
    pub fn resample(&mut self) {
        let head = Segment { curvature: Curvature::Linear, position: Vec2::new(0., 0.) };

        let tail = iter_once(&head)
            .chain(self.path.iter())
            .tuple_windows::<(_, _)>()
            .scan(r32(0.), |state, (prev, curr)| Some(curr.sample(state, prev.position)))
            .flatten();

        self.lut = iter_once(Sample::Point { position: Vec2::new(0., 0.), displacement: r32(0.) })
            .chain(tail)
            .collect::<Vec<_>>();
    }
}

#[derive(Component)]
pub struct SplineIndexCache {
    lut: usize,
    waypoints: usize,
}

impl AutomationClip for SmartSpline {
    type ClipCache = SplineIndexCache;
    type Output = Vec2;
    fn play(&self, clip_time: R32, cache: &mut Self::ClipCache) -> Self::Output {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
