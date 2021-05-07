use std::usize;

use crate::utils::from_end::FromEnd;
use lyon_geom::{CubicBezierSegment, Point, QuadraticBezierSegment};

pub enum CtrlVariant {
    Quadratic(Point<f32>),
    CubicEase(Point<f32>),
    Cubic(Point<f32>, Point<f32>),
}

pub struct Segment {
    start: Point<f32>,
    ctrls: CtrlVariant,
}

pub enum SampleDescriptor {
    S, 
    X,
}

pub struct CurveChain {
    segments: Vec<Segment>,
    segment_samples: Vec<Vec<Point<f32>>>,
    descriptor_values: Vec<Vec<f32>>,
    descriptor: SampleDescriptor,
}

impl CurveChain {
    fn distance(p0: &Point<f32>, p1: &Point<f32>) -> f32 {
        (p0.to_vector() - p1.to_vector()).length()
    }

    fn resample_segment(&mut self, index: usize) {
        assert!(index < self.segments.len() - 1);

        let segments = &self.segments;

        let (p0, p1, p2, op3) = match segments[index].ctrls {
            CtrlVariant::Quadratic(a1) => (
                segments[index].start,
                a1,
                segments[index + 1].start,
                None,
            ),
            CtrlVariant::Cubic(a1, a2) => {
                let v1 = a1.to_vector() + segments[index].start.to_vector();
                let v2 = a2.to_vector() + segments[index + 1].start.to_vector();
                (
                    segments[index].start,
                    Point::new(v1.x, v1.y),
                    Point::new(v2.x, v2.y),
                    Some(segments[index + 1].start),
                )
            }
            CtrlVariant::CubicEase(aX) => {
                let p = segments[index + 1].start - segments[index].start;
                let p12 = Point::new(aX.x * p.x, aX.y * p.y);
                (
                    segments[index].start,
                    p12,
                    p12,
                    Some(segments[index + 1].start),
                )
            }
        };

        let sample = &mut self.segment_samples[index];
        let descriptor = &mut self.descriptor_values[index];

        sample.clear();
        descriptor.clear();
        sample.push(self.segments[index].start);
        descriptor.push(0.0);

        let desc = &self.descriptor; //have to do this to beat borrow checker

        let mut callback = |p: Point<f32>| {
            let d = match desc {
                SampleDescriptor::S => {
                    (p.to_vector() - sample[FromEnd(0)].to_vector()).length() + descriptor[FromEnd(0)]
                }
                SampleDescriptor::X => p.x - segments[index].start.x,
            };
            descriptor.push(d);
            sample.push(p);
        };

        match op3 {
            Some(p3) => {
                CubicBezierSegment::<f32> {
                    from: p0,
                    ctrl1: p1,
                    ctrl2: p2,
                    to: p3,
                }
                .for_each_flattened(0.05, &mut callback);
            }
            None => {
                QuadraticBezierSegment::<f32> {
                    from: p0,
                    ctrl: p1,
                    to: p2,
                }
                .for_each_flattened(0.05, &mut callback);
            }
        }
    }

    pub fn push(&mut self, segment: Segment) {
        self.segments.push(segment);
        self.segment_samples.push(vec![]);
        self.resample_segment(self.segments.len() - 2);
    }

    pub fn pop(&mut self) {
        self.segments.pop();
        self.segment_samples.pop();
    }

    pub fn insert(&mut self, index: usize, segment: Segment) {
        assert!(index <= self.segments.len());
        if index == self.segments.len() {
            self.push(segment);
        }
        else {
            self.segments.insert(index, segment);
            self.resample_segment(index - 1);
            self.resample_segment(index);
        } 
    }

    pub fn remove(&mut self, index: usize) {
        assert!(1 < index && index < self.segments.len());

        if index == self.segments.len() - 1 {
            self.pop();
        }
        else {
            self.segments.remove(index);
            self.segment_samples.remove(index);
            self.resample_segment(index - 1);
        }
    }
}
