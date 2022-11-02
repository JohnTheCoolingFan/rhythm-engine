use crate::utils::*;

use core::iter::once as iter_once;

use bevy::{
    math::f64::{DMat3, DVec2},
    prelude::*,
};
use itertools::Itertools;
use lyon_geom::*;
use noisy_float::prelude::*;
use tap::Pipe;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleKind {
    Point,
    CWArc,
    CCArc,
}

impl SampleKind {
    fn signum(self) -> f64 {
        match self {
            Self::CWArc => 1.,
            Self::CCArc => -1.,
            Self::Point => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct Sample {
    displacement: P64,
    position: DVec2,
    kind: SampleKind,
}

impl Quantify for Sample {
    fn quantify(&self) -> P64 {
        self.displacement
    }
}

impl Lerp for Sample {
    type Output = DVec2;

    #[rustfmt::skip]
    fn lerp(&self, next: &Self, t: T64) -> Self::Output {
        debug_assert!(self.kind == SampleKind::Point);

        match next.kind {
            SampleKind::Point => self.position.lerp(next.position, t.raw()),
            SampleKind::CWArc | SampleKind::CCArc => {
                (self.displacement.raw() - next.displacement.raw())
                    .pipe(|arc_length| arc_length * t.raw() / next.position.distance(self.position))
                    .pipe(|raw_radians| r64(raw_radians * next.kind.signum()))
                    .pipe(|radians| self.position.rotate_about(next.position, radians))
            },
        }
    }
}

#[derive(Clone, Copy)]
pub enum Curvature {
    Linear,
    Circular(DVec2),
    Quadratic(DVec2),
    Cubic(DVec2, DVec2),
}

pub struct Segment {
    curvature: Curvature,
    position: DVec2,
}

#[rustfmt::skip]
impl Segment {
    fn sample_bezier(
        path_length: &mut P64,
        start: DVec2,
        points: impl Iterator<Item = Point<f64>>
    )
        -> Vec<Sample>
    {
        [start]
            .into_iter()
            .chain(points.map(|p| DVec2::new(p.x, p.y)))
            .tuple_windows::<(_, _)>()
            .map(|(prev, curr)| {
                *path_length += prev.distance(curr);
                Sample {
                    position: curr,
                    displacement: *path_length,
                    kind: SampleKind::Point
                }
            })
            .collect::<Vec<_>>()
    }

    fn sample(&self, path_length: &mut P64, start: DVec2) -> Vec<Sample> {
        match self.curvature {
            Curvature::Linear => {
                *path_length += start.distance(self.position);
                vec![Sample {
                    position: self.position,
                    displacement: *path_length,
                    kind: SampleKind::Point
                }]
            },
            Curvature::Circular(ctrl) => {
                let end = self.position;
                // https://math.stackexchange.com/a/1460096
                let m11_determinant = [start, ctrl, end]
                    .map(|point| [point.x, point.y, 1.])
                    .pipe_ref(DMat3::from_cols_array_2d)
                    .transpose()
                    .determinant();

                if m11_determinant.abs() <= f64::EPSILON {
                    *path_length += start.distance(self.position);
                    vec![Sample {
                        position: end,
                        displacement: *path_length,
                        kind: SampleKind::Point
                    }]
                } else {
                    let m12 = [start, ctrl, end]
                        .map(|point| [point.x.powi(2) + point.y.powi(2), point.y, 1.])
                        .pipe_ref(DMat3::from_cols_array_2d)
                        .transpose();

                    let m13 = [start, ctrl, end]
                        .map(|point| [point.x.powi(2) + point.y.powi(2), point.x, 1.])
                        .pipe_ref(DMat3::from_cols_array_2d)
                        .transpose();

                    let center = DVec2::new(
                        0.5 * (m12.determinant() / m11_determinant),
                        -0.5 * (m13.determinant() / m11_determinant),
                    );

                    let (center_to_start, center_to_end, ctrl_dir, arc_dir) = (
                        center - start,
                        center - end,
                        [start, ctrl, end].into_iter().orientation(),
                        [start, center, end].into_iter().orientation()
                    );

                    let theta = (center_to_start.length() * center_to_end.length())
                        .pipe(|denominator| center_to_start.dot(center_to_end) / denominator)
                        .acos()
                        .abs()
                        .pipe(|theta| if ctrl_dir != arc_dir { theta } else { 360. - theta });

                    *path_length += p64(theta * center.distance(start));

                    let samples = [
                        (f64::EPSILON <= center.distance(start)).then(|| Sample {
                            position: center,
                            displacement: *path_length,
                            kind: match ctrl_dir {
                                Orientation::CounterClockWise => SampleKind::CCArc,
                                Orientation::ClockWise => SampleKind::CWArc,
                                _ => unreachable!()
                            }
                        }),
                        Some(Sample {
                            position: end,
                            kind: SampleKind::Point,
                            displacement: *path_length,
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

#[derive(Component, Default)]
pub struct Spline {
    pub path: Vec<Segment>,
    pub lut: Vec<Sample>,
}

#[rustfmt::skip]
impl Spline {
    pub fn resample(&mut self) {
        let start = Segment { curvature: Curvature::Linear, position: DVec2::new(0., 0.) };

        let head = Sample {
            position: DVec2::new(0., 0.),
            displacement: p64(0.),
            kind: SampleKind::Point
        };

        let tail = iter_once(&start)
            .chain(self.path.iter())
            .tuple_windows::<(_, _)>()
            .scan(p64(0.), |state, (prev, curr)| Some(curr.sample(state, prev.position)))
            .flatten();

        self.lut = iter_once(head)
            .chain(tail)
            .collect::<Vec<_>>();
    }

    pub fn play(&self, t: T64) -> DVec2 {
        self.lut
            .last()
            .map(|sample| sample.quantify())
            .filter(|length| f64::EPSILON < length.raw())
            .map_or(DVec2::default(), |length| self
                .lut
                .interp(length * t.raw())
                .unwrap_or_else(|sample| sample.position)
            )
    }
}

#[cfg(test)]
mod tests {
    use super::{super::*, *};
    use Curvature::*;

    #[test]
    #[rustfmt::skip]
    fn play_spline() {
        let mut spline = Spline {
            lut: vec![],
            path: vec![
                Segment {
                    curvature: Linear,
                    position: DVec2::new(2., 0.),
                },
                Segment {
                    curvature: Linear,
                    position: DVec2::new(-1., 0.),
                },
                Segment {
                    curvature: Circular(DVec2::new(0., 1.)),
                    position: DVec2::new(1., 0.),
                },
                Segment {
                    curvature: Circular(DVec2::new(0., 1.)),
                    position: DVec2::new(-1., 0.),
                },
                Segment {
                    curvature: Cubic(DVec2::new(-1., -1.), DVec2::new(0., 0.)),
                    position: DVec2::new(0., -1.),
                },
            ],
        };

        spline.resample();

        let (length, qcw_turn) = (
            spline.lut.last().unwrap().quantify(),
            std::f64::consts::PI / 2.
        );

        let automation = Automation(tiny_vec![
            Anchor::default(),
            Anchor { x: length, val: t64(1.), weight: Weight::Quadratic(r64(0.)) }
        ]);

        let covals = [
            ((0.5, 0.), 0.5),
            ((1., 0.), 1.),
            ((1.5, 0.), 1.5),
            ((1.5, 0.), 2.5),
            ((1., 0.), 3.),
            ((-1., 0.), 5.),
            ((0., 1.), qcw_turn + 5.),
            ((1., 0.), 2. * qcw_turn + 5.),
            ((0., 1.), 3. * qcw_turn + 5.),
            ((-1., 0.), 4. * qcw_turn + 5.),
            ((-0.5, -0.5), (4. * qcw_turn + 5.).pipe(|prev| prev + 0.5 * (length.raw() - prev))),
            ((0., -1.), spline.lut.last().unwrap().quantify().raw() + 1.)
        ];

        covals.iter().for_each(|((x, y), displacement)| {
            let position = spline.play(automation.play(ClampedTime::new(p64(*displacement))));
            let expected = DVec2::new(*x, *y);
            let distance = position.distance(expected);
            assert!(
                distance < 0.001,
                "Input: {displacement}
                Expected: {expected}
                Position: {position}
                Distance: {distance}"
            )
        })
    }
}
