use super::{automation::*, Modulation, Synth};
use crate::utils::*;

use core::iter::once as iter_once;
use std::f32::consts::PI;

use bevy::prelude::*;
use itertools::Itertools;
use lyon_geom::*;
use noisy_float::prelude::*;
use tap::Pipe;

#[derive(Debug)]
pub enum Sample {
    Point { displacement: P32, position: Vec2 },
    Arc { meta: R32, center: Vec2 },
}

impl Sample {
    #[rustfmt::skip]
    fn inner(&self) -> Vec2 {
        match self {
            Self::Point { position: point, .. } | Self::Arc { center: point, .. } => *point,
        }
    }
}

impl Quantify for Sample {
    fn quantify(&self) -> P32 {
        match self {
            Self::Point { displacement, .. } => *displacement,
            Self::Arc { meta, .. } => p32(meta.raw().abs()),
        }
    }
}

impl Lerp for Sample {
    type Output = Vec2;
    #[rustfmt::skip]
    fn lerp(&self, next: &Self, t: T32) -> Self::Output {
        match (self, next) {
            // Arc
            (
                Sample::Point { position: start, displacement },
                Sample::Arc { center, meta },
            ) => {
                (meta.raw().abs() - displacement.raw())
                    .pipe(|arc_length| arc_length * t.raw() / center.distance(*start))
                    .to_degrees()
                    .pipe(|deg| center.rotate(start, r32(deg * meta.raw().signum())))
            },

            // No Arc
            (
                Sample::Point { position: start, .. },
                Sample::Point { position: end, .. }
            ) => {
                start.lerp(*end, t.raw())
            }

            // Shold not happen
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
                        (ctrl_ori, center_ori, theta) if ctrl_ori != center_ori => theta.abs(),
                        (ctrl_ori, _, theta) => (360. - theta.abs()) * ctrl_ori.signum()
                    };

                    *path_length += p32(
                        2. * PI * ((theta.abs() * center.distance(start)).abs() / 360.)
                    );

                    let samples = [
                        (f32::EPSILON <= center.distance(start)).then(|| Sample::Arc {
                            meta: r32(path_length.raw() * theta.signum()),
                            center,
                        }),
                        Some(Sample::Point {
                            displacement: *path_length,
                            position: end,
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
    fn play(&self, offset: P32, lower_clamp: T32, upper_clamp: T32) -> Modulation {
        self.lut
            .last()
            .map(|sample| sample.quantify())
            .filter(|length| f32::EPSILON < length.raw())
            .map_or(Modulation::Nil, |length| self
                .automation
                .interp(offset)
                .unwrap_or_else(|anchor| anchor.val)
                .pipe(|offset| t32((offset / length).raw()))
                .pipe(|offset| p32(lower_clamp.lerp(&upper_clamp, offset).raw()) * length)
                .pipe(|displacement| self.lut.interp(displacement).unwrap_or_else(Sample::inner))
                .pipe(Modulation::Position)
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
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
            .map(Quantify::quantify)
            .map(|length| Anchor { x: length, val: length, weight: Weight::Quadratic(r32(0.)) })
            .map(|anchor| tiny_vec![Anchor::default(), anchor])
            .map(Automation)
            .unwrap();

        let covals = [
            ((0.5, 0.), 0.5),
            ((1., 0.), 1.),
            ((1.5, 0.), 1.5),
            ((1.5, 0.), 2.5),
            ((1., 0.), 3.),
            ((-1., 0.), 5.),
            //((0., 1.), std::f32::consts::PI / 2. + 5.)
        ];

        covals.iter().for_each(|((x, y), input)| {
            if let Position(position) = spline.play(p32(*input), t32(0.), t32(1.)) {
                let expected = Vec2::new(*x, *y);
                let distance = position.distance(expected);
                assert!(
                    distance < 0.001,
                    "Input[{input}] Expected[{expected}] Position[{position}] Distance[{distance}]"
                )
            } else {
                panic!("Unexpected Nill")
            }
        })
    }
}
