use super::{automation::*, Modulation, Synth};
use crate::utils::*;

use core::iter::once as iter_once;
use std::f32::consts::PI;

use bevy::prelude::*;
use itertools::Itertools;
use lyon_geom::*;
use noisy_float::prelude::*;
use tap::Pipe;

#[derive(Debug, Clone, Copy)]
pub enum SampleKind {
    Point,
    CWArc,
    CCArc,
}

impl SampleKind {
    fn signum(self) -> f32 {
        match self {
            Self::CWArc => 1.,
            Self::CCArc => -1.,
            Self::Point => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct Sample {
    displacement: P32,
    position: Vec2,
    kind: SampleKind,
}

impl Quantify for Sample {
    fn quantify(&self) -> P32 {
        self.displacement
    }
}

impl Lerp for Sample {
    type Output = Vec2;

    #[rustfmt::skip]
    fn lerp(&self, next: &Self, t: T32) -> Self::Output {
        match (self.kind, next.kind) {
            (SampleKind::Point, SampleKind::Point) => self.position.lerp(next.position, t.raw()),
            (SampleKind::Point, SampleKind::CWArc | SampleKind::CCArc)  => {
                (self.displacement.raw() - next.displacement.raw())
                    .pipe(|arc_length| arc_length * t.raw() / next.position.distance(self.position))
                    .pipe(|raw_radians| r32(raw_radians * next.kind.signum()))
                    .pipe(|radians| self.position.rotate_about(next.position, radians))
            },
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
                Sample {
                    position: curr,
                    displacement: *path_length,
                    kind: SampleKind::Point
                }
            })
            .collect::<Vec<_>>()
    }

    fn sample(&self, path_length: &mut P32, start: Vec2) -> Vec<Sample> {
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
                    .into_matrix()
                    .determinant();

                if m11_determinant.abs() <= f32::EPSILON {
                    *path_length += start.distance(self.position);
                    vec![Sample {
                        position: end,
                        displacement: *path_length,
                        kind: SampleKind::Point
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

                    let (center_to_start, center_to_end, ctrl_orientation) = (
                        center - start,
                        center - end,
                        [start, ctrl, end].into_iter().orientation()
                    );

                    let theta = (center_to_start.length() * center_to_end.length())
                        .pipe(|denominator| center_to_start.dot(center_to_end) / denominator)
                        .acos()
                        .abs()
                        .pipe(|theta| match [start, center, end].into_iter().orientation() {
                            orientation if orientation != ctrl_orientation => theta,
                            _ => (360. - theta)
                        });

                    *path_length += p32(theta * center.distance(start));

                    let samples = [
                        (f32::EPSILON <= center.distance(start)).then(|| Sample {
                            displacement: *path_length,
                            position: center,
                            kind: match ctrl_orientation {
                                Orientation::CounterClockWise => SampleKind::CCArc,
                                Orientation::ClockWise => SampleKind::CWArc,
                                _ => unreachable!()
                            }
                        }),
                        Some(Sample {
                            displacement: *path_length,
                            position: end,
                            kind: SampleKind::Point
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

        let lut_start = Sample {
            position: Vec2::new(0., 0.),
            displacement: p32(0.),
            kind: SampleKind::Point
        };

        self.lut = iter_once(lut_start)
            .chain(tail)
            .collect::<Vec<_>>();
    }
}

impl Synth for Spline {
    type Output = Modulation;

    #[rustfmt::skip]
    fn play(&self, offset: P32, lower_clamp: T32, upper_clamp: T32) -> Modulation {
        self.lut
            .last()
            .map(|sample| sample.quantify())
            .filter(|length| f32::EPSILON < length.raw())
            .map_or(Modulation::Nil, |length| self
                .automation
                .interp(offset)
                .unwrap_or_else(|anchor| anchor.val)
                .pipe(|raw_displacement| t32((raw_displacement / length).raw()))
                .pipe(|ratio| p32(lower_clamp.lerp(&upper_clamp, ratio).raw()) * length)
                .pipe(|displacement| self.lut .interp(displacement))
                .unwrap_or_else(|sample| sample.position)
                .pipe(Modulation::Position)
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tinyvec::*;
    use Curvature::*;
    use Modulation::*;

    #[test]
    #[rustfmt::skip]
    fn play_spline() {
        let mut spline = Spline {
            lut: vec![],
            automation: Automation(tiny_vec![]),
            path: vec![
                Segment {
                    curvature: Linear,
                    position: Vec2::new(2., 0.),
                },
                Segment {
                    curvature: Linear,
                    position: Vec2::new(-1., 0.),
                },
                Segment {
                    curvature: Circular(Vec2::new(0., 1.)),
                    position: Vec2::new(1., 0.),
                },
                Segment {
                    curvature: Circular(Vec2::new(0., 1.)),
                    position: Vec2::new(-1., 0.),
                },
                Segment {
                    curvature: Cubic(Vec2::new(-1., -1.), Vec2::new(0., 0.)),
                    position: Vec2::new(0., -1.),
                },
            ],
        };

        spline.resample();

        spline.automation = spline
            .lut
            .last()
            .unwrap()
            .quantify()
            .pipe(|length| Anchor { x: length, val: length, weight: Weight::Quadratic(r32(0.)) })
            .pipe(|anchor| tiny_vec![Anchor::default(), anchor])
            .pipe(Automation);

        let covals = [
            ((0.5, 0.), 0.5),
            ((1., 0.), 1.),
            ((1.5, 0.), 1.5),
            ((1.5, 0.), 2.5),
            ((1., 0.), 3.),
            ((-1., 0.), 5.),
            ((0., 1.), std::f32::consts::PI / 2. + 5.),
            ((1., 0.), std::f32::consts::PI + 5.),
            ((0., 1.), 3. * std::f32::consts::PI / 2. + 5.),
            ((-1., 0.), 2. * std::f32::consts::PI + 5.),
            ((-0.5, -0.5), (2. * std::f32::consts::PI + 5.).pipe(|prev|
                prev + 0.5 * (spline.lut.last().unwrap().quantify().raw() - prev)
            )),
            ((0., -1.), spline.lut.last().unwrap().quantify().raw() + 1.)
        ];

        covals.iter().for_each(|((x, y), input)| {
            if let Position(position) = spline.play(p32(*input), t32(0.), t32(1.)) {
                let expected = Vec2::new(*x, *y);
                let distance = position.distance(expected);
                assert!(
                    distance < 0.001,
                    "Input: {input} Expected: {expected} Position: {position} Distance: {distance}"
                )
            } else {
                panic!("Unexpected Nill")
            }
        })
    }
}
