use std::{marker::PhantomData, usize};

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


pub struct CurveChain {
    segments: Vec<Segment>,
    segment_samples: Vec<Vec<Point<f32>>>,
    segment_descriptions: Vec<Vec<f32>>,
    descriptor: fn(&CurveChain, usize, &Point<f32>) -> f32
}

impl CurveChain {
    fn by_s(curve: &CurveChain, index: usize, point: &Point<f32>) -> f32 {
        let samples = &curve.segment_samples[index];
        let descriptions = &curve.segment_descriptions[index];
        (point.to_vector() - samples[FromEnd(0)].to_vector()).length() + descriptions[FromEnd(0)]
    }

    fn by_x(curve: &CurveChain, index: usize, point: &Point<f32>) -> f32 {
        point.x - curve.segments[index].start.x
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
            },
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

        self.segment_samples[index].clear();
        self.segment_descriptions[index].clear();
        self.segment_samples[index].push(self.segments[index].start);
        self.segment_descriptions[index].push(0.0);

        let mut callback = |p: Point<f32>| {
            let d = (self.descriptor)(self, index, &p);
            self.segment_samples[index].push(p);
            self.segment_descriptions[index].push(d);
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
