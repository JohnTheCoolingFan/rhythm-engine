use crate::utils::from_end::FromEnd;
use glam::f32::Mat3;
use lyon_geom::{CubicBezierSegment, Point, QuadraticBezierSegment};

#[derive(Clone, Copy)]
pub enum CtrlVariant {
    Linear(Point<f32>),
    Quadratic(Point<f32>, Point<f32>),
    ThreePointCircle(Point<f32>, Point<f32>),
    Cubic(Point<f32>, Point<f32>, Point<f32>),
}

impl CtrlVariant { 
    fn end(&self) -> Point<f32> {
        match self {
            CtrlVariant::Linear(p) => { *p },
            CtrlVariant::Quadratic(_, p) => { *p },
            CtrlVariant::ThreePointCircle(_, p) => { *p },
            CtrlVariant::Cubic(_, _, p) => { *p }
        }
    }
}

pub struct Segment {
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
    pub fn new(desc: fn(&CurveChain, usize, &Point<f32>) -> f32) -> Self {
        Self {
            segments: vec![Segment{ ctrls: CtrlVariant::Linear(Point::new(0.0, 0.0)), tolerence: 0.0 }],
            segment_samples: vec![vec![]],
            segment_descriptions: vec![vec![]],
            descriptor: desc,
        }
    }

    pub fn displacement_desctiptor(curve: &CurveChain, index: usize, point: &Point<f32>) -> f32 {
        debug_assert!(index < curve.segment_samples.len());
        let samples = &curve.segment_samples[index];
        let descriptions = &curve.segment_descriptions[index];

        descriptions[FromEnd(0)] + (point.to_vector() - samples[FromEnd(0)].to_vector()).length()
    }

    pub fn monotonic_x_descriptor(curve: &CurveChain, index: usize, point: &Point<f32>) -> f32 {
        debug_assert!(index < curve.segment_samples.len());
        let x = curve.segments[index].ctrls.end().x;

        let diff = point.x - x;
        debug_assert!(0.0 <= diff);
        return diff;
    }

    fn resample_segment(&mut self, index: usize) {
        debug_assert!(index + 1 < self.segments.len());
        debug_assert!(index < self.segment_samples.len());
        debug_assert!(index < self.segment_descriptions.len());

        let start = self.segments[index].ctrls.end();
        let end = self.segments[index + 1].ctrls.end();

        self.segment_samples[index].clear();
        self.segment_descriptions[index].clear();
        self.segment_samples[index].push(start);
        self.segment_descriptions[index].push(0.0);

        let tolerence = self.segments[index].tolerence;
        let ctrls = self.segments[index].ctrls;
        //have to pull these out because the closure captures self

        let mut callback = |p: Point<f32>| {
            let d = (self.descriptor)(self, index, &p);
            self.segment_samples[index].push(p);
            self.segment_descriptions[index].push(d);
        };
        //can't use self below closure

        match ctrls {
            CtrlVariant::Linear(_) => {
                callback(end);
            }
            CtrlVariant::Quadratic(c, _) => {
                QuadraticBezierSegment::<f32> {
                    from: start,
                    ctrl: c,
                    to: end,
                }
                .for_each_flattened(tolerence, &mut callback);
            }
            #[rustfmt::skip]
            CtrlVariant::ThreePointCircle(c, _) => {
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
            CtrlVariant::Cubic(a1, a2, _) => {
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

#[cfg(test)]
mod tests {
    use super::*;
    use ggez::event::{self, EventHandler, KeyCode, KeyMods, MouseButton};
    use ggez::graphics;
    use ggez::{Context, GameResult};
    use glam::*;

    struct CurveTest {
        curve: CurveChain,
        point_buff: [Point<f32>; 3],
        insert_index: usize,
        selected_segment: Option<usize>
    }

    impl CurveTest {
        fn new() -> GameResult<CurveTest> {
            Ok(CurveTest {
                curve: CurveChain::new(
                    CurveChain::displacement_desctiptor,
                ),
                point_buff: [Point::new(0.0, 0.0); 3],
                insert_index: 0,
                selected_segment: None
            })
        }
    }

    impl EventHandler for CurveTest {
        fn update(&mut self, ctx: &mut Context) -> GameResult {

            Ok(())
        }

        fn draw(&mut self, ctx: &mut Context) -> GameResult {
            Ok(())
        }

        fn key_down_event(&mut self, ctx: &mut Context, key: KeyCode, mods: KeyMods, _: bool) {
            if key == KeyCode::Escape { self.selected_segment = None; return; }

            let points = &self.point_buff;
            let segment = Segment{ tolerence: 0.05, ctrls: match key {
                KeyCode::Key1 => { CtrlVariant::Linear(points[FromEnd(0)]) },
                KeyCode::Key2 => { CtrlVariant::Quadratic(points[FromEnd(1)], points[FromEnd(0)]) },
                KeyCode::Key3 => { CtrlVariant::ThreePointCircle(points[FromEnd(1)], points[FromEnd(0)]) },
                KeyCode::Key4 => { CtrlVariant::Cubic(points[0], points[1], points[2]) }
                _ => { return; }
            }};
            self.insert_index = 0;

            match self.selected_segment {
                None => { self.curve.push(segment); },
                Some(index) => { self.curve.insert(index, segment); }
            }
        }

        fn mouse_button_down_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
            match button {
                MouseButton::Left => { 
                    self.point_buff[self.insert_index] = Point::new(x, y);
                    self.insert_index = (self.insert_index + 1) % 3;
                }
                _ => {}
            }
        }
    }

    #[test]
    fn curve_chain_test() {
        let cb = ggez::ContextBuilder::new("Curve test", "iiYese");
        let (ctx, event_loop) = cb.build().unwrap();
        let state = CurveTest::new().unwrap();
        event::run(ctx, event_loop, state);
    }
}
