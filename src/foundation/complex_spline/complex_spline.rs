use crate::foundation::{automation::*, complex_spline::*};
use crate::utils::*;
use duplicate::duplicate;
use glam::Vec2;
use lyon_geom::Point;

pub struct ComplexSpline {
    anchors: Vec<Anchor>,
    segments: Vec<Segment>,
}

pub struct Critical {
    pub anchor: Anchor,
    pub segment: Segment
}

impl ComplexSpline {
    fn new(len: f32, initial: Segment) -> Self {
        let mut new = Self {
            anchors: vec![
                Anchor::new(Vec2::new(0., 0.)),
                Anchor::new(Vec2::new(len, 1.))
            ],
            segments: vec![
                Segment::new(Ctrl::Linear(Point::new(0., 0.)), 0.),
                initial
            ]
        };

        let p = &new.segments[0].ctrls.get_end();
        new.segments[1].recompute(p);
        new
    }

    fn resample(&mut self, index: usize) {
        assert!(index != 0);
        let (p0, p1) = (
            &self.segments[index - 1].ctrls.get_end(),
            &self.segments[index].ctrls.get_end()
        );
        self.segments[index].recompute(p0);
        if index + 1 < self.segments.len() {
            self.segments[index + 1].recompute(p1);
        }
    }

    fn correct_anchor(&mut self, index: usize) {
        assert!((1..self.anchors.len()).contains(&index));
        self.anchors[index].point.y = if index % 2 == 0 { 0. } else { 1. };

        self.anchors[index].point.x = self.anchors[index].point.x.clamp(
            self.anchors[index - 1].point.x,
            if index + 1 < self.anchors.len() {
                self.anchors[index + 1].point.x
            }
            else {
                f32::MAX
            }
        );
    }
 
    //can't use index for the parallel vectors because that requires GATs
    //which at the time of writing this code is unstable
    pub fn anchors(&self) -> &Vec<Anchor> {
        &self.anchors
    }
    
    pub fn segments(&self) -> &Vec<Segment> {
        &self.segments
    }

    pub fn insert(&mut self, Critical{anchor, segment}: Critical) -> usize {
        let index = self.anchors.quantified_insert(anchor);
        self.segments.insert(index, segment);
        self.correct_anchor(index);
        self.resample(index);
        index
    }
 
    pub fn remove(&mut self, index: usize) -> Critical {
        self.segments[index].lut.clear();
        let removed = Critical{
            anchor: self.anchors.remove(index),
            segment: self.segments.remove(index)
        };
        self.resample(index);
        removed
    }

    pub fn modify<Func>(&mut self, index: usize, mut func: Func)
        -> Result<Critical, ()>
    where
        Func: FnMut(&mut Anchor, &mut Segment)
    {
        if (1..self.segments.len()).contains(&index) {
            let old = Critical{
                anchor: self.anchors[index],
                segment: self.segments[index].yoink()
            };

            func(&mut self.anchors[index], &mut self.segments[index]);

            self.correct_anchor(index);
            if old.segment.ctrls != self.segments[index].ctrls {
                self.resample(index);
            }

            Ok(old)
        }
        else {
            Err(())
        }
    }

    pub fn insert_anchor(&mut self, index: usize, anch: Anchor) -> Result<Anchor, ()> {
        if !(1..self.anchors.len()).contains(&index.debug("index")) {
            Err(())
        }
        else {
            let start = self.segments[index - 1].ctrls.get_end();
            let end = self.segments[index].ctrls.get_end();
            self.segments[index].ctrls = Ctrl::Linear(
                start 
                + ((end.to_vector() - start.to_vector())
                * (1. / 2.))
            );
            self.segments.insert(
                index + 1,
                Segment::new(Ctrl::Linear(end), 0.5)
            );
            self.resample(index);
            Ok(self.anchors.quantified_emplace(index, anch,
                |a, min, max| {
                    a.point.x = a.point.x.clamp(
                        min.unwrap_or(0.),
                        max.unwrap_or(f32::MAX)
                    );
                    a.point.y = if index % 2 == 0 { 0. } else { 1. }
                }
            ))
        }
    }

    pub fn emplace_segment(&mut self, anch_pos: f32, seg: Segment) -> usize {
        assert!(0. < anch_pos);
        let index = self.anchors.quantified_insert(Anchor::new(Vec2::new(anch_pos, 0.)));
        self.segments.insert(index, seg);
        self.correct_anchor(index);
        self.resample(index);
        index
    }
 
    pub fn closest_segment(&self, point: Point<f32>) -> usize {
        let index = self.segments
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                (a.ctrls.get_end() - point).length()
                    .partial_cmp(&(b.ctrls.get_end() - point).length())
                    .unwrap()
            })
            .unwrap().0;

        if index == 0 { 1 } else { index }
    }

    pub fn closest_anchor(&self, x: f32) -> usize {
        let index = self.anchors
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                (a.point.x - x).abs()
                    .partial_cmp(&(b.point.x - x).abs())
                    .unwrap()
            })
            .unwrap().0;

        if index == 0 { 1 } else { index }
    }

}
//
//
//
//
//
type CompSplSeeker<'a> = Seeker<&'a Vec<Segment>, (BPSeeker<'a, Anchor>, SegmentSeeker<'a>)>;

impl<'a> SeekerTypes for CompSplSeeker<'a> {
    type Source = <BPSeeker<'a, Anchor> as SeekerTypes>::Source;
    type Output = Vec2;
}

impl<'a> Seek for CompSplSeeker<'a> {
    #[duplicate(method; [seek]; [jump])]
    fn method(&mut self, offset: f32) -> Vec2 {
        let (ref mut anchorseeker, ref mut lutseeker) = self.meta;
        let old = anchorseeker.index();
        let mut t = anchorseeker.method(offset);
        let new = anchorseeker.index();

        if new % 2 != 0 {
            t = t.lerp_invert()
        }

        if old != new && !anchorseeker.over_run() {
            *lutseeker = self.data[new].seeker();
        }

        lutseeker.method(t)
    }
}

impl<'a> Seekable<'a> for ComplexSpline {
    type Seeker = CompSplSeeker<'a>;
    fn seeker(&'a self) -> Self::Seeker {
        Self::Seeker {
            data: &self.segments,
            meta: (self.anchors.seeker(), self.segments[0].seeker())
        }
    }
}
//
//
//
//
//
#[cfg(test)]
mod tests {
    use super::*;
    use ggez::{
        event::{self, EventHandler, KeyCode, KeyMods, MouseButton},
        graphics::*,
        input::keyboard::is_key_pressed,
        timer::time_since_start,
        Context, GameResult, GameError
    };
    use lyon_geom::Point;

    #[derive(Debug)]
    enum Selection {
        Segment(usize),
        Anchor(usize),
        None
    }

    struct Test {
        cmpspl: ComplexSpline,
        point_buff: Vec<Point<f32>>,
        dimensions: Vec2,
        selection: Selection,
    }

    impl Test {
        fn new() -> GameResult<Self> {
            let x = 2000.;
            let y = 1000.;
            Ok(Self {
                cmpspl: ComplexSpline::new(x, Segment::new(Ctrl::Linear(Point::new(x, 0.)), 1.)),
                point_buff: vec![],
                dimensions: Vec2::new(x, y),
                selection: Selection::None,
            })
        }
    }

    impl EventHandler<GameError> for Test {
        fn update(&mut self, _ctx: &mut Context) -> GameResult {
            Ok(())
        }

        fn draw(&mut self, ctx: &mut Context) -> GameResult {
            //
            //  setup
            //
            clear(ctx, Color::new(0., 0., 0., 1.));
            let mouse_pos: Vec2 = ggez::input::mouse::position(ctx).into();

            let circle = Mesh::new_circle(
                ctx,
                DrawMode::fill(),
                Vec2::new(0.0, 0.0),
                10.0,
                2.0,
                Color::new(1.0, 1.0, 1.0, 1.0),
            )?;
            draw(ctx, &circle, (mouse_pos,))?;

            let rect = Mesh::new_rectangle(
                ctx,
                DrawMode::fill(),
                Rect::new(-5., -5., 10., 10.),
                Color::new(1., 0., 0., 0.5),
            )?;

            //
            //  time
            //
            let t = self.dimensions.x * (
                (time_since_start(ctx).as_millis() as f32 % 5000.)
                / 5000.
            );

            let t_line = MeshBuilder::new()
                .polyline(
                    DrawMode::Stroke(StrokeOptions::DEFAULT),
                    &[
                        Vec2::new(0., self.dimensions.y * 0.75),
                        Vec2::new(0., self.dimensions.y),
                    ],
                    Color::WHITE,
                )?
                .build(ctx)?;

            draw(ctx, &t_line, (Vec2::new(t, 0.),))?;
            draw(ctx, &circle, (self.cmpspl.seeker().jump(t),))?;

            //
            //  automation
            //
            let mut anch_seeker = self.cmpspl.anchors().seeker();
            let res = 200;
            let auto_points: Vec<Vec2> = (0..res)
                .map(|x| {
                    Vec2::new(
                        (x as f32 / res as f32) * self.dimensions.x,
                        self.dimensions.y - (self.dimensions.y / 4.)
                        * anch_seeker.seek((x as f32 / res as f32) * self.dimensions.x),
                    )
                })
                .collect();

            let auto_lines = MeshBuilder::new()
                .polyline(
                    DrawMode::Stroke(StrokeOptions::DEFAULT),
                    auto_points.as_slice(),
                    Color::new(1., 1., 1., 1.),
                )?
                .build(ctx)?;
            draw(ctx, &auto_lines, (Vec2::new(0.0, 0.0),))?;

            //
            //  spline
            //
            for i in 0..self.cmpspl.segments().len() {
                let segment = &self.cmpspl.segments()[i];
                match segment.ctrls {
                    Ctrl::Linear(p) => {
                        draw(ctx, &circle, (Vec2::new(p.x, p.y),))?;
                    }
                    Ctrl::Quadratic(p1, p2) => {
                        draw(ctx, &rect, (Vec2::new(p1.x, p1.y),))?;
                        draw(ctx, &circle, (Vec2::new(p2.x, p2.y),))?;
                    }
                    Ctrl::Cubic(p1, p2, p3) => {
                        let start = self.cmpspl.segments()[i - 1].ctrls.get_end();
                        draw(ctx, &rect, (Vec2::new(start.x + p1.x, start.y + p1.y),))?;
                        draw(ctx, &rect, (Vec2::new(p2.x + p3.x, p2.y + p3.y),))?;
                        draw(ctx, &circle, (Vec2::new(p3.x, p3.y),))?;
                    }
                    Ctrl::ThreePointCircle(p1, p2) => {
                        draw(ctx, &rect, (Vec2::new(p1.x, p1.y),))?;
                        draw(ctx, &circle, (Vec2::new(p2.x, p2.y),))?;
                    }
                }
            }

            for i in 1..self.cmpspl.segments().len() {
                let mut points = Vec::<Vec2>::new();
                let mut seeker = self.cmpspl.segments()[i].seeker();
                let mut t = 0.;
                while t <= 1. {
                    points.push(seeker.seek(t));
                    t += 0.05;
                }
                points.push(seeker.seek(1.));

                let lines = MeshBuilder::new()
                    .polyline(
                        DrawMode::Stroke(StrokeOptions::DEFAULT),
                        points.as_slice(),
                        Color::new(1.0, 1.0, 1.0, 1.0),
                    )?
                    .build(ctx)?;
                draw(ctx, &lines, (Vec2::new(0.0, 0.0),))?;
            }

            present(ctx)?;
            Ok(())
        }

        fn mouse_button_down_event(
            &mut self,
            _ctx: &mut Context,
            button: MouseButton,
            x: f32,
            y: f32,
        ) {
            match button {
                MouseButton::Left => {
                    if self.dimensions.y * 0.75 < y {
                        let index = self.cmpspl.closest_anchor(x);
                        if (self.cmpspl.anchors()[index].point.x - x).abs() < 10. {
                            self.selection = Selection::Anchor(index);
                        } else {
                            match self.selection {
                                Selection::Anchor(n) | Selection::Segment(n) => {
                                    self.cmpspl.modify(n, |anch, _| {
                                        anch.point.x = x
                                    }).unwrap();
                                },
                                _ => {
                                    self.cmpspl.insert_anchor(index, Anchor::new(Vec2::new(x, 0.))).unwrap();
                                }
                            }
                        }
                    } else {
                        let index = self.cmpspl.closest_segment(Point::new(x, y));
                        let dist = (self.cmpspl.segments[index].ctrls.get_end() - Point::new(x, y)).length();
                        if dist < 10. {
                            self.selection = Selection::Segment(index);
                        }
                        else {
                            match self.selection {
                                Selection::None => self.point_buff.push(Point::new(x, y)),
                                Selection::Anchor(n) | Selection::Segment(n) => {
                                    self.cmpspl.modify(n, |_, segment| {
                                        segment.ctrls.set_end(Point::new(x, y));
                                    }).unwrap();
                                }    
                            }
                        }
                    }
                },
                MouseButton::Middle => {
                    if let Selection::Anchor(n) | Selection::Segment(n) = self.selection {
                        self.cmpspl.modify(n, |anch, _| {
                            anch.weight.cycle();
                        }).unwrap();
                    }
                }
                _ => {}
            }
        }

        fn key_down_event(&mut self, _ctx: &mut Context, key: KeyCode, _mods: KeyMods, _: bool) {
            let points = &self.point_buff;
            let ctrls = match key {
                KeyCode::Key1 => Ctrl::Linear(points[FromEnd(0)]),
                KeyCode::Key2 => Ctrl::Quadratic(points[FromEnd(1)], points[FromEnd(0)]),
                KeyCode::Key3 => Ctrl::ThreePointCircle(points[FromEnd(1)], points[FromEnd(0)]),
                KeyCode::Key4 => Ctrl::Cubic(points[0], points[1], points[2]),
                KeyCode::Escape => {
                    self.selection = Selection::None;
                    return;
                }
                _ => {
                    if let Selection::Anchor(index) = self.selection {
                        automation::tests::key_handle(&mut self.cmpspl.anchors[index], key);
                    }
                    return;
                }
            };

            if let Selection::Segment(index) = self.selection {
                self.cmpspl.modify(index, |_, seg| seg.ctrls = ctrls).unwrap();
                self.point_buff.clear();
            }
        }

        fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, y: f32) {
            if let Selection::Anchor(index) | Selection::Segment(index) = self.selection {
                self.cmpspl.modify(index, 
                    |anchor, _| { 
                        anchor.weight.shift_power(if 0. < y { 0.05 } else { -0.05 }).unwrap();
                    }
                ).unwrap();
            }
        }
    }

    #[test]
    fn cspline() -> GameResult {
        let state = Test::new()?;
        let cb = ggez::ContextBuilder::new("Complex Spline test", "iiYese").window_mode(
            ggez::conf::WindowMode::default().dimensions(state.dimensions.x, state.dimensions.y),
        );
        let (ctx, event_loop) = cb.build()?;
        event::run(ctx, event_loop, state)
    }
}
