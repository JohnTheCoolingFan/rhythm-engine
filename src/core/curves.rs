use crate::utils::from_end::FromEnd;
use glam::f32::Mat3;
use lyon_geom::{CubicBezierSegment, Point, QuadraticBezierSegment};

pub enum CtrlVariant {
    Linear,
    Quadratic(Point<f32>),
    ThreePointCircle(Point<f32>),
    Cubic(Point<f32>, Point<f32>),
}

pub struct Segment {
    start: Point<f32>,
    ctrls: CtrlVariant,
    tolerence: f32,
}

pub struct CurveChain {
    segments: Vec<Segment>,
    segment_samples: Vec<Vec<Point<f32>>>,
    segment_descriptions: Vec<Vec<f32>>,
    descriptor: fn(&CurveChain, usize, &Point<f32>) -> f32,
}

impl CurveChain {
    pub fn new() -> Self {}

    pub fn displacement_desctiptor(curve: &CurveChain, index: usize, point: &Point<f32>) -> f32 {
        debug_assert!(index < curve.segment_samples.len());
        let samples = &curve.segment_samples[index];
        let descriptions = &curve.segment_descriptions[index];
        (point.to_vector() - samples[FromEnd(0)].to_vector()).length() + descriptions[FromEnd(0)]
    }

    pub fn monotonic_x_descriptor(curve: &CurveChain, index: usize, point: &Point<f32>) -> f32 {
        debug_assert!(index < curve.segment_samples.len());
        let x = point.x - curve.segments[index].start.x;
        debug_assert!(curve.segment_descriptions[index][FromEnd(0)] <= x);
        return x;
    }

    fn resample_segment(&mut self, index: usize) {
        debug_assert!(index < self.segment_samples.len());
        debug_assert!(index < self.segment_descriptions.len());

        self.segment_samples[index].clear();
        self.segment_descriptions[index].clear();
        self.segment_samples[index].push(self.segments[index].start);
        self.segment_descriptions[index].push(0.0);

        let mut callback = |p: Point<f32>| {
            let d = (self.descriptor)(self, index, &p);
            self.segment_samples[index].push(p);
            self.segment_descriptions[index].push(d);
        };

        let segments = &self.segments;
        let (start, end) = (&segments[index].start, &segments[index + 1].start);

        match segments[index].ctrls {
            CtrlVariant::Linear => {
                callback(self.segments[index + 1].start);
            }
            CtrlVariant::Quadratic(c) => {
                QuadraticBezierSegment::<f32> {
                    from: *start,
                    ctrl: c,
                    to: *end,
                }
                .for_each_flattened(0.05, &mut callback);
            }
            #[rustfmt::skip]
            CtrlVariant::ThreePointCircle(c) => {
                let m11 = Mat3::from_cols_array(&[
                    start.x, start.y, 1.,
                    c.x    , c.y    , 1.,
                    end.x  , end.y  , 1.
                ]).transpose();

                let m12 = Mat3::from_cols_array(&[
                    start.x.powi(2) + start.y.powi(2), start.y, 1.,
                    c.x.powi(2)     + c.y.powi(2)    , c.y    , 1.,
                    end.x.powi(2)   + end.y.powi(2)  , end.y  , 1.,
                ])
                .transpose();

                let m13 = Mat3::from_cols_array(&[
                    start.x.powi(2) + start.y.powi(2), start.x, 1.,
                    c.x.powi(2)     + c.y.powi(2)    , c.x    , 1.,
                    end.x.powi(2)   + end.y.powi(2)  , end.x  , 1.
                ]).transpose();

                let x = 0.5 * (m12.determinant()/m11.determinant());
                let y = -0.5 * (m13.determinant()/m11.determinant());
            }
            CtrlVariant::Cubic(a1, a2) => {
                CubicBezierSegment::<f32> {
                    from: *start,
                    ctrl1: Point::new(a1.x + start.x, a1.y + start.y), //they're different point types
                    ctrl2: Point::new(a2.x + end.x, a2.y + end.y), //so no common addition interface
                    to: *end,
                }
                .for_each_flattened(0.05, &mut callback)
            }
        };
    }

    pub fn push(&mut self, segment: Segment) {
        self.segments.push(segment);
        self.segment_samples.push(vec![]);
        self.segment_descriptions.push(vec![]);

        self.resample_segment(self.segments.len() - 2);
    }

    pub fn pop(&mut self) {
        self.segments.pop();
        self.segment_samples.pop();
        self.segment_descriptions.pop();
    }

    pub fn insert(&mut self, index: usize, segment: Segment) {
        assert!(index <= self.segments.len());
        if index == self.segments.len() {
            self.push(segment);
        } else {
            self.segments.insert(index, segment);
            self.resample_segment(index - 1);
            self.resample_segment(index);
        }
    }

    pub fn remove(&mut self, index: usize) {
        assert!(1 < index && index < self.segments.len());

        if index == self.segments.len() - 1 {
            self.pop();
        } else {
            self.segments.remove(index);
            self.segment_samples.remove(index);
            self.resample_segment(index - 1);
        }
    }
}
