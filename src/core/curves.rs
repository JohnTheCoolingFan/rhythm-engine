use std::usize;

use lyon_geom::{CubicBezierSegment, Point, QuadraticBezierSegment};

pub enum AnchorCtrl {
    Quadratic(Point<f32>),
    CubicEase(Point<f32>),
    Cubic(Point<f32>, Point<f32>)
}

struct Anchor {
    start: Point<f32>,
    ctrls: AnchorCtrl
}

pub enum Sampler {
    S,
    X
}

pub struct Displacement(f32);
pub struct VectorX(f32);

struct CurveChain {
    segment_points: Vec<Anchor>,
    point_samples: Vec<Vec<Point<f32>>>,
    value_samples: Vec<Vec<f32>>,
    val_sampler: BezierSampler
}

impl CurveChain {
    fn distance(p0: &Point<f32>, p1: &Point<f32>) {
        (p0.to_vector() - p1.to_vector()).length()
    }

    fn sample_segment(&mut self, index: usize) {
        assert!(index < self.points.len() - 1);

        let seg_points = &self.segment_points;

        let (p0, p1, p2, op3) = match seg_points[index].ctrls {
            AnchorCtrl::Quadratic(P1) => {
                (seg_points[index].start, 
                 P1, 
                 seg_points[index + 1].start, 
                 None)
            },
            AnchorCtrl::Cubic(P1, P2) => {
                (seg_points[index].start,
                 P1 + seg_points[index].start, 
                 P2 + seg_points[index + 1].start,
                 Some(seg_points[index + 1].start))
            }
            AnchorCtrl::CubicEase(PX) => {
                let p = seg_points[index + 1].start - seg_points[index].start; 
                let p12 = Point::new(PX.x * p.x, PX.y * p.y); 
                
                (seg_points[index].start, 
                 p12, 
                 p12, 
                 Some(seg_points[index + 1].start))
            }
        }

        let points = &self.point_samples[index];
        let values = &self.value_samples[index];
        
        points.clear();
        values.clear();
        points.push(self.points[index].start);
        values.push(0.0);
        match op3 {
            Some(p3) => {
                let segment = CubicBezierSegment::<f32> {
                    from: p0,
                    ctrl1: p1,
                    ctrl2: p2,
                    to: p3
                };

                segment.for_each_flattened(0.05, &mut |p| {
                    values.push( match self.val_sampler { 
                        BezierSampler::S => 0.0,
                        BezierSampler::X => p.x
                    });
                });
            },
            None => {
                let segment = QuadraticBezierSegment::<f32> {
                    from: p0,
                    ctrl: p1,
                    to: p2
                };
            }
        }
    }
}
