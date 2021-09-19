use super::{automation::*, segment::*};
use crate::utils::*;
use glam::Vec2;
use lyon_geom::Point;
use tinyvec::tiny_vec;
use duplicate::duplicate;

type SegmentMetaSeeker<'a> = Seeker<&'a [Epoch<Segment>], usize>;

impl<'a> SegmentMetaSeeker<'a> {
    pub fn point_from_s(&self, virtual_s: f32) -> Vec2 {
        match self.data.seeker().jump(virtual_s) {
            0 => self.data[1].val.seeker().seek(0.),
            oob if self.data.len() <= oob => {
                let p = self.data[FromEnd(0)].val.ctrls.end();
                Vec2::new(p.x, p.y)
            },
            index => {
                let segment = &self.data[index];
                let mut real_s = virtual_s - self.data[index - 1].offset;
                if let Ctrl::ThreePointCircle(_, _) = segment.val.ctrls {
                    real_s /= segment.val.length();
                }
                segment.val.seeker().jump(real_s)
            }
        }
    }
}

impl<'a> SeekerTypes for SegmentMetaSeeker<'a> {
    type Source = Epoch<Segment>;
    type Output = usize;    // Segment shouldn't be Copy and this avoids dealing with lifetimes
}

impl<'a> Exhibit for SegmentMetaSeeker<'a> {
    fn exhibit(&self, _: f32) -> Self::Output {
        self.meta
    }
}
//
//
//
//
//
pub struct ComplexSpline {
    pub automation: Automation<f32>,
    segments: TVec<Epoch<Segment>>
}

impl ComplexSpline {
    pub fn new(auto_len: f32, initial: Point<f32>) -> Self {
        let mut cmpspl =  Self {
            automation: Automation::new(0., initial.to_vector().length(), auto_len),
            segments: tiny_vec!([Epoch<_>; SHORT_ARR_SIZE] =>
                Epoch {
                    offset: 0.,
                    val: Segment::new(Ctrl::Linear(Point::new(0., 0.)), 0.5)
                },
                Epoch {
                    offset: 0.,
                    val: Segment::new(Ctrl::Linear(initial), 0.5)
                }
            )
        };

        cmpspl.resample(1);
        cmpspl.remeasure(1);
        cmpspl
    }

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
        assert!(from != 0);
        self.segments[from].offset = self.segments[from].val.length();
        if 1 < from { self.segments[from].offset += self.segments[from - 1].offset }

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
            if *index < self.segments.len() { func(&mut self.segments[*index].val); }
            else { err.push(*index) }
        }

        for index in selection {
            if *index < self.segments.len() { self.resample(*index); }
        }

        let min = *selection.iter().min().unwrap();
        
        if min < self.segments.len() { self.remeasure(min); }
        else { self.remeasure(1); }

        if err.is_empty() { Ok(()) } else { Err(err) }
    }

    pub fn split_segment(&mut self, index: usize) {
        assert!(index != 0);

        self.segments.insert(index,
            Epoch {
                offset: 0.,
                val: Segment::new(
                        Ctrl::Linear(self.segments[index - 1].val.ctrls.end().lerp(
                            *self.segments[index].val.ctrls.end(), 0.5
                        )),
                    0.5
                )
            }
        );

        self.resample(index);
        self.resample(index + 1);
        self.remeasure(1);
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

    #[duplicate(
        bound_limit_at      bound;
        [lower_limit_at]    [lower];
        [upper_limit_at]    [upper]
    )]
    pub fn bound_limit_at(&self, t: f32) -> Vec2 {
        let s = self.automation.bound.seeker().jump(t).val;
        self.segments.seeker().point_from_s(s)
    }
}
//
//
//
//
// 
type CompSplSeeker<'a> = Seeker<&'a [Epoch<Segment>], AutomationSeeker<'a, f32>>;

impl<'a> SeekerTypes for CompSplSeeker<'a> {
    type Source = <AutomationSeeker<'a, f32> as SeekerTypes>::Source;
    type Output = Vec2;
}

impl<'a> Seek for CompSplSeeker<'a> { 
    fn jump(&mut self, t: f32) -> Vec2 {
        let s = self.meta.jump(t);
        self.data.seeker().point_from_s(s)
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
}
//
//
//
//
//
#[cfg(test)]
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
    use super::super::Anchor;

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
                cmpspl: ComplexSpline::new(x, Point::new(x , 0.)),
                point_buff: vec![],
                dimensions: Vec2::new(x, y),
                selection: Selection::None
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

            let circle = Mesh::new_circle(ctx,
                DrawMode::fill(),
                Vec2::new(0.0, 0.0),
                10.0, 2.0,
                Color::new(1.0, 1.0, 1.0, 1.0),
            )?;
            let red_circle = Mesh::new_circle(ctx,
                DrawMode::fill(),
                Vec2::new(0.0, 0.0),
                10.0, 2.0,
                Color::new(1.0, 0., 0., 1.0),
            )?;
            let blue_circle = Mesh::new_circle(ctx,
                DrawMode::fill(),
                Vec2::new(0.0, 0.0),
                10.0, 2.0,
                Color::new(0., 0., 1., 1.0),
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

            //
            //  automation
            //
            let mut anch_seeker = self.cmpspl.automation.anchors.seeker();
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
            for i in 1..self.cmpspl.segments.len() {
                let segment = &self.cmpspl.segments[i];
                match segment.val.ctrls {
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

            for i in 1..self.cmpspl.segments.len() {
                let points: Vec<Vec2> = match self.cmpspl.segments[i].val.ctrls {
                    Ctrl::ThreePointCircle(_, _) => {
                        (0..=20)
                            .map(|n|
                                 self.cmpspl.segments[i].val.seeker().seek(n as f32 / 20.)
                            ).collect()
                    }
                    _ => self.cmpspl.segments[i].val.lut
                        .iter()
                        .map(|e| e.val)
                        .collect()
                };
                    
                let lines = MeshBuilder::new()
                    .polyline(
                        DrawMode::Stroke(StrokeOptions::DEFAULT),
                        points.as_slice(),
                        Color::new(1.0, 1.0, 1.0, 1.0),
                    )?
                    .build(ctx)?;
                draw(ctx, &lines, (Vec2::new(0.0, 0.0),))?;
            }

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
            draw(ctx, &blue_circle, (self.cmpspl.lower_limit_at(t),))?;
            draw(ctx, &red_circle, (self.cmpspl.seeker().jump(t),))?;
            draw(ctx, &blue_circle, (self.cmpspl.upper_limit_at(t),))?;

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
                        let index = self.cmpspl.automation.closest_anchor(Vec2::new(x, y));
                        if (self.cmpspl.automation.anchors[index].point.x - x).abs() < 10. {
                            self.selection = Selection::Anchor(index);
                        } else {
                            match self.selection {
                                Selection::Anchor(n) | Selection::Segment(n) => {
                                    self.cmpspl.automation.modify_anchors(&[n], |anch| {
                                        anch.point.x = x
                                    }).unwrap();
                                },
                                _ => {
                                    self.cmpspl.automation.insert_anchor(
                                        Anchor::new(
                                            Vec2::new(
                                                x,
                                                1. - (y - self.dimensions.y * 0.75) / (self.dimensions.y * 0.25)
                                            )
                                        )
                                    );
                                }
                            }
                        }
                    } else {
                        let index = self.cmpspl.closest_segment(
                            Point::new(x, y)
                        );
                        let dist = (
                            self.cmpspl.segments[index].val.ctrls.end().to_vector()
                            - Point::new(x, y).to_vector()
                        ).length();
                        if dist < 10. {
                            self.selection = Selection::Segment(index);
                        }
                        else {
                            match self.selection {
                                Selection::None => self.point_buff.push(Point::new(x, y)),
                                Selection::Segment(n) => {
                                    self.cmpspl.modify_segments(&[n], |segment| {
                                        *segment.ctrls.end_mut() = Point::new(x, y);
                                    }).unwrap();
                                }
                                _ => {}
                            }
                        }
                    }
                },
                MouseButton::Middle => {
                    match self.selection {
                        Selection::Anchor(n) => {
                            self.cmpspl.automation.modify_anchors(&[n], |anch| {
                                anch.weight.cycle();
                            }).unwrap();
                        },
                        Selection::Segment(n) => {
                            self.cmpspl.split_segment(n);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        fn key_down_event(&mut self, _ctx: &mut Context, key: KeyCode, _mods: KeyMods, _: bool) {
            if let KeyCode::Escape = key {
                self.selection = Selection::None;
                return;
            }

            if let Selection::Anchor(index) = self.selection {
                automation::tests::key_handle(&mut self.cmpspl.automation.anchors[index], key);
                return;
            }
                    
            let points = &self.point_buff;
            let ctrls = match key {
                KeyCode::Key1 => Ctrl::Linear(points[FromEnd(0)]),
                KeyCode::Key2 => Ctrl::Quadratic(points[FromEnd(1)], points[FromEnd(0)]),
                KeyCode::Key3 => Ctrl::ThreePointCircle(points[FromEnd(1)], points[FromEnd(0)]),
                KeyCode::Key4 => Ctrl::Cubic(points[0], points[1], points[2]),
                _ => {
                    return;
                }
            };

            if let Selection::Segment(index) = self.selection {
                self.cmpspl.modify_segments(&[index], |seg| seg.ctrls = ctrls).unwrap();
                self.point_buff.clear();
            }
        }

        fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, y: f32) {
            if let Selection::Anchor(index) = self.selection {
                self.cmpspl.automation.modify_anchors(&[index], 
                    |anchor| { 
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
}
