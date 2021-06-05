use std::f32::consts::PI;

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
    point_lut: Vec<Vec2>,
    value_lut: Vec<f32>,
}

impl Segment {
    pub fn new(ctrl_type: Ctrl, segment_tolerence: f32) -> Self {
        Self {
            ctrls: ctrl_type,
            tolerence: segment_tolerence,
            point_lut: vec![],
            value_lut: vec![]
        }
    }

    pub fn recompute(&mut self, start: Point<f32>) {
        let end = self.ctrls.end();

        self.point_lut.clear();
        self.value_lut.clear();
        self.point_lut.push(Vec2::new(start.x, start.y));
        self.value_lut.push(0.0);

        //these are the only variables used after the colsure definition
        let ctrls = self.ctrls;
        let tolerence = self.tolerence;

        let mut callback = |p: Point<f32>| {
            let last = self.point_lut[FromEnd(0)];
            let s = self.value_lut[FromEnd(0)]
                + (p.to_vector() - Point::new(last.x, last.y).to_vector()).length();
            self.point_lut.push(Vec2::new(p.x, p.y));
            self.value_lut.push(s);
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
            #[rustfmt::skip]
            Ctrl::ThreePointCircle(c, end) => {
                self.value_lut.clear();
                //https://math.stackexchange.com/a/1460096
                let m11 = Mat3::from_cols_array(&[
                    start.x, start.y, 1.,
                    c.x    , c.y    , 1.,
                    end.x  , end.y  , 1.
                ]).transpose();

                let d11 = m11.determinant();

                println!("{}", d11);
                if 0. != d11 {
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

                    let center =  Point::new(
                        0.5 * (m12.determinant()/d11),
                        -0.5 * (m13.determinant()/d11)
                    );
                   
                    self.point_lut.push(Vec2::new(center.x, center.y));
                    
                    let side = c.is_left(&start, &end);
                    let rot_sign = match start.rotate_about(&center, 1.).is_left(&start, &end) == side {
                        true => 1.,
                        false => -1.
                    };

                    let center_to_start = start - center;
                    let center_to_end = end - center;

                    let theta = (
                        center_to_start.dot(center_to_end) 
                        / (center_to_start.length() * center_to_end.length())
                    ).acos() * 180. / PI;

                    let angle = 
                        if (start.rotate_about(&center, theta * rot_sign) - end).length() < 0.01 { 
                            theta
                        }
                        else {
                            360. - theta
                        };

                    self.value_lut.push(angle * rot_sign);
                }
                else {
                    self.point_lut.push(Vec2::new(end.x, end.y));
                }
            }
        };

        match ctrls {
            Ctrl::ThreePointCircle(_, _) => {}
            _ => {
                let max_displ = self.value_lut[FromEnd(0)];
                for elem in &mut self.value_lut { *elem /= max_displ; }
            }
        }
    }

    pub fn get_point_lut(&self) -> &Vec<Vec2> {
        &self.point_lut
    }

    pub fn get_value_lut(&self) -> &Vec<f32> {
        &self.value_lut
    }
}

pub struct SegmentSeeker<'a> {
    index: usize,
    segment: &'a Segment
}

impl <'a> SegmentSeeker<'a> {
    fn interp(&self, t: f32) -> Vec2 {
        debug_assert!(0 < self.index && self.index < self.segment.value_lut.len());
        debug_assert!(
            self.segment.value_lut[self.index - 1] <= t 
            && t <= self.segment.value_lut[self.index]
        );
        let start = self.segment.point_lut[self.index - 1];
        let end = self.segment.point_lut[self.index];

        let s = (t - self.segment.value_lut[self.index - 1]) 
            / (self.segment.value_lut[self.index] - self.segment.value_lut[self.index - 1]);

        end * s + start * (1. - s)
    }
}

impl <'a> Seeker<Vec2> for SegmentSeeker<'a> {
    fn seek(&mut self, t: f32) -> Vec2 {
        debug_assert!(0. <= t && t <= 1.);
        match self.segment.ctrls {
            Ctrl::ThreePointCircle(_, _) => {
                if self.segment.value_lut.len() == 0 {
                    self.segment.point_lut[0].lerp(self.segment.point_lut[1], t)
                }
                else{ 
                    self.segment.point_lut[0].rotate_about( 
                        &self.segment.point_lut[1], 
                        self.segment.value_lut[0] * t
                    )
                }
            }
            _ => {
                while self.index < self.segment.value_lut.len() {
                    if t == self.segment.value_lut[self.index] { 
                        return self.segment.point_lut[self.index];
                    } else if t < self.segment.value_lut[self.index] {
                        break;
                    }
                    self.index += 1;
                }

                if 0 == self.index {
                    self.segment.point_lut[self.index]
                }
                else {
                    self.interp(t)
                }
            }
        }
    }

    fn jump(&mut self, t: f32) -> Vec2 {
        debug_assert!(0. <= t && t <= 1.);
        match self.segment.ctrls {
            Ctrl::ThreePointCircle(_, _) => {
                self.seek(t)
            }
            _ => {
                match self.segment.value_lut.binary_search_by(
                    |v| v.partial_cmp(&t).unwrap()
                ) {
                    Ok(index) => {
                        self.index = index;
                        self.segment.point_lut[index]
                    }
                    Err(index) => {
                        self.index = index;
                        if 0 == index || index == self.segment.point_lut.len() {
                            self.segment.point_lut[index]
                        }
                        else {
                            self.interp(t)
                        }
                    }
                }
            }
        }
    }
}

impl <'a> Seekable<'a, Vec2> for Segment {
    type Seeker = SegmentSeeker<'a>;
    fn seeker(&'a self) -> Self::Seeker {
        SegmentSeeker{ index: 0, segment: &self }
    }
}
