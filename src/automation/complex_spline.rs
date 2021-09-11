use super::{automation::*, segment::*};
use crate::utils::*;
use glam::Vec2;
use lyon_geom::Point;

/*impl SeekerTypes for Epoch<Segment> {
}*/

pub struct ComplexSpline {
    pub automation: Automation<f32>,
    segments: TVec<Epoch<Segment>>
}

impl ComplexSpline {
    fn resample(&mut self, index: usize) {
        assert!(index != 0);
        let (p0, p1) = (
            *self.segments[index - 1].val.ctrls.end(),
            *self.segments[index].val.ctrls.end()
        );
        self.segments[index].val.recompute(&p0);
        if index + 1 < self.segments.len() {
            self.segments[index + 1].val.recompute(&p1);
        }
    }

    fn remeasure(&mut self, from: usize) {
        self.segments[from].offset = self.segments[from].val.length();
        if 0 < from { self.segments[from].offset += self.segments[from - 1].offset }

        for i in (from + 1)..self.segments.len() {
            self.segments[i].offset = self.segments[i - 1].offset + self.segments[i].val.length()
        }
    }

    pub fn modify_segments<Func>(&mut self, selection: &[usize], mut func: Func)
        -> Result<(), Vec<usize>>
    where
        Func: FnMut(&mut Segment)
    {
        let mut err: Vec<usize> = vec![];
        
        for index in selection {
            if self.segments.len() < *index {
                func(&mut self.segments[*index].val);
            }
            else {
                err.push(*index)
            }
        }

        for index in selection {
            if self.segments.len() < *index {
                self.resample(*index);
            }
        }

        self.remeasure(*selection.iter().min().unwrap());

        if err.is_empty() { Ok(()) } else { Err(err) }
    }

    pub fn insert_segment(&mut self, index: usize, seg: Segment) {
        self.segments.insert(index, Epoch{ offset: 0., val: seg});
        self.resample(index);
    }
 
    pub fn closest_segment(&self, point: Point<f32>) -> usize {
        let index = self.segments
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                (a.val.ctrls.end().to_vector() - point.to_vector()).length()
                    .partial_cmp(&(b.val.ctrls.end().to_vector() - point.to_vector()).length())
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
type CompSplSeeker<'a> = Seeker<&'a TVec<Epoch<Segment>>, AutomationSeeker<'a, f32>>;

impl<'a> SeekerTypes for CompSplSeeker<'a> {
    type Source = <AutomationSeeker<'a, f32> as SeekerTypes>::Source;
    type Output = Vec2;
}

impl<'a> SeekerTypes for Seeker<&'a TVec<Epoch<Segment>>, usize>
{
    type Source = Epoch<Segment>;
    type Output = &'a Segment;
}

impl<'a> Exhibit for Seeker<&'a TVec<Epoch<Segment>>, usize> {
    fn exhibit(&self, _: f32) -> Self::Output {
        match self.current(){
            Ok(curr) | Err(curr) => &curr.val
        }
    }
}

/*impl<'a> Seek for CompSplSeeker<'a> { 
    fn jump(&mut self, t: f32) -> Vec2 {
        let s = self.meta.seek(t);
        let segment = self.data.seeker().jump(s);

    }

    fn seek(&mut self, t: f32) -> Vec2 {
        self.jump(t)
    }
}

impl<'a> Seekable<'a> for ComplexSpline {
    type Seeker = CompSplSeeker<'a>;
    fn seeker(&'a self) -> Self::Seeker {
        Self::Seeker {
            data: &self.segments,
            meta: self.automation.seeker()
        }
    }
}*/
//
//
//
//
//
/*#[cfg(test)]
mod tests {
    use super::*;
    use super::super::automation;
    use ggez::{
        event::{self, EventHandler, KeyCode, KeyMods, MouseButton},
        graphics::*,
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
            draw(ctx, &circle, (self.cmpspl.seeker().seek(t),))?;

            //
            //  automation
            //
            let mut anch_seeker = self.cmpspl.anchors().seeker();
            let res = self.dimensions.x as i32;
            let auto_points: Vec<Vec2> = (0..res)
                .map(|x| {
                    Vec2::new(
                        (x as f32 / res as f32) * self.dimensions.x,
                        self.dimensions.y - (
                            0.25 
                            * self.dimensions.y
                            * anch_seeker.seek((x as f32 / res as f32) * self.dimensions.x)
                        )
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
                        draw(ctx, &rect, (Vec2::new(p1.x, p1.y),))?;
                        draw(ctx, &rect, (Vec2::new(p2.x, p2.y),))?;
                        draw(ctx, &circle, (Vec2::new(p3.x, p3.y),))?;
                    }
                    Ctrl::ThreePointCircle(p1, p2) => {
                        draw(ctx, &rect, (Vec2::new(p1.x, p1.y),))?;
                        draw(ctx, &circle, (Vec2::new(p2.x, p2.y),))?;
                    }
                }
            }

            for i in 1..self.cmpspl.segments().len() {
                let points: Vec<Vec2> = self
                    .cmpspl
                    .segments()[i]
                    .lut
                    .iter()
                    .map(|e| e.val)
                    .collect();

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
                                    self.cmpspl.insert_anchor(Anchor::new(Vec2::new(x, 0.)));
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
            if let Selection::Anchor(index) = self.selection {
                automation::tests::key_handle(&mut self.cmpspl.anchors[index], key);
                return;
            }
                    
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
                        let _ = anchor.weight.shift_curvature(
                            if 0. < y { 0.05 } else { -0.05 }
                        );
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
}*/
