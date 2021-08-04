use std::f32::consts::PI;

use crate::utils::*;
use duplicate::duplicate;
use glam::{f32::Mat3, Vec2};
use lyon_geom::{CubicBezierSegment, Point, QuadraticBezierSegment};

#[derive(Clone, Copy, Debug)]
pub enum Ctrl {
    Linear(Point<f32>),
    Quadratic(Point<f32>, Point<f32>),
    ThreePointCircle(Point<f32>, Point<f32>),
    Cubic(Point<f32>, Point<f32>, Point<f32>),
}

impl Ctrl {
    pub fn get_end(&self) -> Point<f32> {
        match self {
            Self::Linear(p) => *p,
            Self::Quadratic(_, p) => *p,
            Self::ThreePointCircle(_, p) => *p,
            Self::Cubic(_, _, p) => *p,
        }
    }

    pub fn set_end(&mut self, point: Point<f32>) -> Point<f32> {
        let p = match self {
            Self::Linear(ref mut p) => p,
            Self::Quadratic(_, ref mut p) => p,
            Self::ThreePointCircle(_, ref mut p) => p,
            Self::Cubic(_, _, ref mut p) => p,
        };
        let old = *p;
        *p = point;
        old
    }
}

pub struct Segment {
    pub ctrls: Ctrl,
    pub tolerence: f32,
    lut: Vec<SimpleAnchor<Vec2>>,
}

impl Segment {
    pub fn new(ctrl_type: Ctrl, segment_tolerence: f32) -> Self {
        Self {
            ctrls: ctrl_type,
            tolerence: segment_tolerence,
            lut: vec![(0., Vec2::new(0., 0.)).into()],
        }
    }

    #[rustfmt::skip]
    pub(super) fn recompute(&mut self, start: Point<f32>) {
        let end = self.ctrls.get_end();

        self.lut.clear();
        self.lut.push((0., Vec2::new(start.x, start.y)).into());

        //these are the only variables used after the colsure definition
        let ctrls = self.ctrls;
        let tolerence = self.tolerence;

        let mut callback = |p: Point<f32>| {
            let last = self.lut[FromEnd(0)].val;
            let s = 
                self.lut[FromEnd(0)].offset
                + (
                    p.to_vector() - Point::new(last.x, last.y).to_vector()
                ).length();
            
            self.lut.push((s, Vec2::new(p.x, p.y)).into());
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
            Ctrl::ThreePointCircle(c, end) => {
                self.lut.clear();
                //https://math.stackexchange.com/a/1460096
                let m11 = Mat3::from_cols_array(&[
                    start.x, start.y, 1.,
                    c.x    , c.y    , 1.,
                    end.x  , end.y  , 1.
                ]).transpose();

                let d11 = m11.determinant();

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

                    self.lut.push((0., Vec2::new(center.x, center.y)).into());

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

                    self.lut.push((angle * rot_sign, Vec2::new(end.x, end.y)).into());
                }
                else {
                    self.lut.push((0., Vec2::new(end.x, end.y)).into());
                }
            }
        };

        match ctrls {
            Ctrl::ThreePointCircle(_, _) => {}
            _ => {
                let max_displ = self.lut[FromEnd(0)].offset;
                for elem in &mut self.lut {
                    elem.offset /= max_displ;
                }
            }
        }
    }
}

pub struct SegmentSeeker<'a> {
    index: usize,
    segment: &'a Segment,
    lut_seeker: <Vec<SimpleAnchor<Vec2>> as Seekable<'a>>::SeekerType,
}

///0. <= t <= 1.
impl<'a> Seeker<Vec2> for SegmentSeeker<'a> {
    #[duplicate(method; [seek]; [jump];)]
    fn method(&mut self, t: f32) -> Vec2 {
        debug_assert!(0. <= t && t <= 1.);
        if let Ctrl::ThreePointCircle(_, _) = self.segment.ctrls {
            if self.segment.lut.len() == 3 {
                self.segment.lut[0]
                    .val
                    .rotate_about(&self.segment.lut[1].val, self.segment.lut[2].offset * t)
            } else {
                self.segment.lut[0].val.lerp(self.segment.lut[1].val, t)
            }
        } else {
            let d = t * self.segment.lut[FromEnd(0)].offset;
            let end = self.lut_seeker.method(d);
            if self.lut_seeker.over_run() | self.lut_seeker.under_run() {
                end
            } else {
                let curr_index = self.lut_seeker.index();
                let prev_index = curr_index - 1;

                let start = self.segment.lut[prev_index].val;

                let s = (d - self.segment.lut[prev_index].offset)
                    / (self.segment.lut[curr_index].offset - self.segment.lut[prev_index].offset);

                start.lerp(end, s)
            }
        }
    }
}

impl<'a> Seekable<'a> for Segment {
    type Output = Vec2;
    type SeekerType = SegmentSeeker<'a>;
    fn seeker(&'a self) -> SegmentSeeker<'a> {
        Self::SeekerType {
            index: 0,
            segment: &self,
            lut_seeker: self.lut.seeker(),
        }
    }
}
