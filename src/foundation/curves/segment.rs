use crate::utils::*;
use glam::{f32::Mat3, Vec2};
use lyon_geom::{CubicBezierSegment, Point, QuadraticBezierSegment};

#[derive(Clone, Copy)]
pub enum Ctrl {
    Linear(Point<f32>),
    Quadratic(Point<f32>, Point<f32>),
    ThreePointCircle(Point<f32>, Point<f32>),
    Cubic(Point<f32>, Point<f32>, Point<f32>),
}

impl Ctrl {
    pub fn end(&self) -> Point<f32> {
        match self {
            Ctrl::Linear(p) => *p,
            Ctrl::Quadratic(_, p) => *p,
            Ctrl::ThreePointCircle(_, p) => *p,
            Ctrl::Cubic(_, _, p) => *p,
        }
    }
}

pub struct Segment {
    pub ctrls: Ctrl,
    pub tolerence: f32,
    samples: Vec<Vec2>,
    displacements: Vec<f32>,
}

struct SegmentSeeker<'a> {
    index: usize,
    segment: &'a Segment
}

impl <'a> Seeker<Vec2> for SegmentSeeker<'a> {
    fn advance(&mut self, offset: f32) -> Vec2 {
        match self.segment.ctrls {
            Ctrl::ThreePointCircle(a1, end) => {
                Vec2::new(0., 0.)
            }
            _ => {
                while self.index < self.segment.displacements.len() {
                    if offset == self.segment.displacements[self.index] { 
                        return self.segment.samples[self.index];
                    } else if offset < self.segment.displacements[self.index] {
                        break;
                    }
                    self.index += 1;
                }
                let s = self.segment.displacements[index - 1]
                self.segment.samples[self.index]
            }
        }
    }

    fn jump(&mut self, offset: f32) -> Vec2 {
        match self.segment.ctrls {
            Ctrl::ThreePointCircle(a1, end) => {
                Vec2::new(0., 0.)
            }
            _ => {
                match self
                    .segment
                    .displacements
                    .binary_search_by(|t| t.partial_cmp(&&offset).unwrap()) {
                        Ok(index) => {
                            self.index = index;
                            self.segment.samples[index]
                        }
                        Err(index) => {
                            self.index = index;
                            if 0 <= index && index + 1 < self.segment.samples.len() {
                                let start = self.segment.samples[index];
                                let end = self.segment.samples[index + 1];
                                let diff = 
                            }
                        }
                }
            }
        }
    }
}

impl Segment {
    pub fn new(ctrl_type: Ctrl, segment_tolerence: f32) -> Self {
        Self {
            ctrls: ctrl_type,
            tolerence: segment_tolerence,
            samples: vec![],
            displacements: vec![]
        }
    }

    pub fn resample(&mut self, start: Point<f32>) {
        let end = self.ctrls.end();

        self.samples.clear();
        self.displacements.clear();
        self.samples.push(Vec2::new(start.x, start.y));
        self.displacements.push(0.0);

        //these are the only variables used after the colsure definition
        let ctrls = self.ctrls;
        let tolerence = self.tolerence;

        let mut callback = |p: Point<f32>| {
            let last = self.samples[FromEnd(0)];
            let d = self.displacements[FromEnd(0)]
                + (p.to_vector() - Point::new(last.x, last.y).to_vector()).length();
            self.samples.push(Vec2::new(p.x, p.y));
            self.displacements.push(d);
        };

        match ctrls {
            Ctrl::Linear(_) => {
                callback(end);
            }
            Ctrl::Quadratic(c, _) => {
                QuadraticBezierSegment::<f32> {
                    from: start,
                    ctrl: c,
                    to: end,
                }
                .for_each_flattened(tolerence, &mut callback);
            }
            Ctrl::Cubic(a1, a2, _) => {
                CubicBezierSegment::<f32> {
                    from: start,
                    ctrl1: Point::new(a1.x + start.x, a1.y + start.y), //they're different point types
                    ctrl2: Point::new(a2.x + end.x, a2.y + end.y), //so no common addition interface
                    to: end,
                }
                .for_each_flattened(tolerence, &mut callback)
            }
            _ => {
                //continuous segments that do not need sampling have their start point
                //placed as the first element of the sample vector
            }
        };

        match ctrls {
            Ctrl::ThreePointCircle(_, _) => {}
            _ => {
                let max_displ = self.displacements[FromEnd(0)];
                for elem in &mut self.displacements { *elem /= max_displ; }
            }
        }
    }

    pub fn get_samples(&self) -> &Vec<Vec2> {
        &self.samples
    }

    pub fn get_displacements(&self) -> &Vec<f32> {
        &self.displacements
    }

    pub fn interpolate(&self, s: f32) {
        let start = &self.samples[0];
        match self.ctrls {
            #[rustfmt::skip]
            Ctrl::ThreePointCircle(c, end) => {
                //https://math.stackexchange.com/a/1460096
                let m11 = Mat3::from_cols_array(&[
                    start.x, start.y, 1.,
                    c.x    , c.y    , 1.,
                    end.x  , end.y  , 1.
                ]).transpose();

                let d11 = m11.determinant();

                if 0.05 < d11 {
                    let m12 = Mat3::from_cols_array(&[
                        start.x.powi(2) + start.y.powi(2), start.y, 1.,
                        c.x.powi(2)     + c.y.powi(2)    , c.y    , 1.,
                        end.x.powi(2)   + end.y.powi(2)  , end.y  , 1.,
                    ]).transpose();

                    let m13 = Mat3::from_cols_array(&[
                        start.x.powi(2) + start.y.powi(2), start.x, 1.,
                        c.x.powi(2)     + c.y.powi(2)    , c.x    , 1.,
                        end.x.powi(2)   + end.y.powi(2)  , end.x  , 1.
                    ]).transpose();

                    let x =  0.5 * (m12.determinant()/d11);
                    let y = -0.5 * (m13.determinant()/d11);
                    //unfinished
                }
                else {
                    //lerp
                }
            }
            _ => {

            }
        }
    }
}


