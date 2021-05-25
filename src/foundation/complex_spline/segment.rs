use crate::utils::from_end::FromEnd;
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
    descriptions: Vec<f32>,
}

impl Segment {
    pub fn new(ctrl_type: Ctrl, segment_tolerence: f32) -> Self {
        Self {
            ctrls: ctrl_type,
            tolerence: segment_tolerence,
            samples: vec![],
            descriptions: vec![],
        }
    }

    pub fn resample(&mut self, start: Point<f32>) {
        let end = self.ctrls.end();

        self.samples.clear();
        self.descriptions.clear();
        self.samples.push(Vec2::new(start.x, start.y));
        self.descriptions.push(0.0);

        //these are the only variables used after the colsure definition
        let ctrls = self.ctrls;
        let tolerence = self.tolerence;

        let mut callback = |p: Point<f32>| {
            let last = self.samples[FromEnd(0)];
            let d = self.descriptions[FromEnd(0)]
                + (p.to_vector() - Point::new(last.x, last.y).to_vector()).length();
            self.samples.push(Vec2::new(p.x, p.y));
            self.descriptions.push(d);
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
            #[rustfmt::skip]
            Ctrl::ThreePointCircle(c, _) => {
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
                    callback(end);
                }
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
        };
    }

    pub fn get_samples(&self) -> &Vec<Vec2> {
        &self.samples
    }

    pub fn get_descriptions(&self) -> &Vec<f32> {
        &self.descriptions
    }
}
