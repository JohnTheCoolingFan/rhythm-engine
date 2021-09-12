use std::f32::consts::PI;
use duplicate::duplicate;
use crate::utils::*;
use glam::{f32::Mat3, Vec2};
use tinyvec::tiny_vec;
use lyon_geom::{CubicBezierSegment, Point, QuadraticBezierSegment};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ctrl {
    Linear(Point<f32>),
    Quadratic(Point<f32>, Point<f32>),
    ThreePointCircle(Point<f32>, Point<f32>),
    Cubic(Point<f32>, Point<f32>, Point<f32>),
}

impl Ctrl {
    pub fn end(&self) -> &Point<f32> {
        match self {
            Self::Linear(p) => p,
            Self::Quadratic(_, p) => p,
            Self::ThreePointCircle(_, p) => p,
            Self::Cubic(_, _, p) => p,
        }
    }
    pub fn end_mut(&mut self) -> &mut Point<f32> {
        match self {
            Self::Linear(p) => p,
            Self::Quadratic(_, p) => p,
            Self::ThreePointCircle(_, p) => p,
            Self::Cubic(_, _, p) => p,
        }
    }
}
//
//
//
//
//
#[derive(Debug)]
pub struct Segment {
    pub ctrls: Ctrl,
    pub tolerence: f32,
    pub(super) lut: TVec<Epoch<Vec2>>,
}

impl Segment {
    pub fn new(ctrl_type: Ctrl, segment_tolerence: f32) -> Self {
        Self {
            ctrls: ctrl_type,
            tolerence: segment_tolerence,
            lut: tiny_vec!([Epoch<Vec2>; SHORT_ARR_SIZE] => (0., Vec2::new(0., 0.)).into()),
        }
    }

    pub fn yoink(&self) -> Self {
        Self {
            ctrls: self.ctrls,
            tolerence: self.tolerence,
            lut: tiny_vec!()
        }
    }

    #[rustfmt::skip]
    pub(super) fn recompute(&mut self, start: &Point<f32>) {
        let end = *self.ctrls.end();

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
                    from: *start,
                    ctrl: c,
                    to: end,
                }
                .for_each_flattened(tolerence, &mut callback);
            }
            Ctrl::Cubic(a1, a2, _) => {
                CubicBezierSegment::<f32> {
                    from: *start,
                    ctrl1: a1,
                    ctrl2: a2,
                    to: end,
                }
                .for_each_flattened(tolerence, &mut callback)
            }
            Ctrl::ThreePointCircle(c, end) => {
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

                    let side = c.is_left(start, &end);
                    let rot_sign = match start.rotate_about(&center, 1.).is_left(start, &end) == side {
                        true => 1.,
                        false => -1.
                    };

                    let center_to_start = *start - center;
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
    }

    pub fn length(&self) -> f32 {
        match self.ctrls {
            Ctrl::ThreePointCircle(_, _) => {
                if self.lut.len() == 3 {
                    self.lut[2].offset.abs().to_radians() * std::f32::consts::PI
                }
                else {
                    (self.lut[0].val - self.lut[1].val).length()
                }
            }
            _ => self.lut[FromEnd(0)].offset
        }
    }
}
//
//
//
//
//
impl Default for Segment {
    fn default() -> Self {
        Self {
            ctrls: Ctrl::Linear(Point::new(0., 0.)),
            tolerence: 0.,
            lut: tiny_vec!()
        }
    }
}

impl<'a> Exhibit for Seeker<&'a TVec<Epoch<Vec2>>, usize> {
    fn exhibit(&self, offset: f32) -> Vec2 {
        match (self.previous(), self.current()) {
            (Some(prev), Ok(curr)) => {
                let t = (offset - prev.offset) / (curr.offset - prev.offset);
                prev.val.lerp(curr.val, t)
            }
            (None, Ok(curr) | Err(curr)) | (_, Err(curr)) => curr.val,
        }
    }
}

pub type SegmentSeeker<'a> = Seeker<&'a Segment, Seeker<&'a TVec<Epoch<Vec2>>, usize>>;

impl<'a> SeekerTypes for SegmentSeeker<'a> {
    type Source = <Seeker<&'a TVec<Epoch<Vec2>>, usize> as SeekerTypes>::Source;
    type Output = Vec2;
}

impl<'a> Seek for SegmentSeeker<'a> {
    #[duplicate(method; [seek]; [jump])]
    fn method(&mut self, t: f32) -> Vec2 {
        if let Ctrl::ThreePointCircle(_, _) = self.data.ctrls {
            if self.data.lut.len() == 3 {
                self.data.lut[0]
                    .val
                    .rotate_about(&self.data.lut[1].val, self.data.lut[2].offset * t)
            } else {
                self.data.lut[0].val.lerp(self.data.lut[1].val, t)
            }
        }
        else {
            self.meta.method(t)
        }
    }
}

impl<'a> Seekable<'a> for Segment {
    type Seeker = SegmentSeeker<'a>;
    fn seeker(&'a self) -> Self::Seeker {
        Self::Seeker {
            data: self,
            meta: self.lut.seeker()
        }
    }
}
