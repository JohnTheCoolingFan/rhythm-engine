use core::iter::once as iter_once;
use std::f32::consts::PI;

use bevy::prelude::*;
use itertools::Itertools;
use lyon_geom::*;
use noisy_float::prelude::*;

use super::{automation::*, repeater::*, Modulation, Synth};
use crate::{hit::*, utils::*};

pub enum Sample {
    Point {
        displacement: P32,
        position: Vec2,
    },
    Arc {
        displacement: P32,
        center: Vec2,
    },
}

impl Quantify for Sample {
    fn quantify(&self) -> P32 {
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
                center.rotate(
                    start,
                    r32((*displacement / center.distance(*start)).to_degrees().raw())
                )
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
        path_length: &mut P32,
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

    fn sample(&self, path_length: &mut P32, start: Vec2) -> Vec<Sample> {
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

                    let displacement = p32(
                        2. * PI * ((theta * center.distance(start)).abs() / 360.)
                    );

                    let samples = [
                        (f32::EPSILON <= center.distance(start)).then(|| Sample::Arc {
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

#[derive(Component)]
pub struct Spline {
    pub path: Vec<Segment>,
    pub lut: Vec<Sample>,
    pub automation: Automation<P32>,
}

impl Spline {
    #[rustfmt::skip]
    pub fn resample(&mut self) {
        let head = Segment { curvature: Curvature::Linear, position: Vec2::new(0., 0.) };

        let tail = iter_once(&head)
            .chain(self.path.iter())
            .tuple_windows::<(_, _)>()
            .scan(p32(0.), |state, (prev, curr)| Some(curr.sample(state, prev.position)))
            .flatten();

        self.lut = iter_once(Sample::Point { position: Vec2::new(0., 0.), displacement: p32(0.) })
            .chain(tail)
            .collect::<Vec<_>>();
    }
}

impl Synth for Spline {
    type Output = Modulation;
    #[rustfmt::skip]
    fn play_from(
        &self,
        offset: P32,
        repetition: Option<Repetition>,
    )
        -> Modulation
    {
        /*Modulation::Position(
            self.lut
                .interp(self.automation.play(time))
                .unwrap_or_else(|sample| match sample {
                    Sample::Arc { center, .. } => *center,
                    Sample::Point { position, .. } => *position,
                }),
        )*/
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
