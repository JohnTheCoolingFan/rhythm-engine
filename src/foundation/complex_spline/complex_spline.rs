use crate::foundation::{automation::*, complex_spline::*};
use crate::utils::*;
use duplicate::duplicate;
use glam::Vec2;
use lyon_geom::Point;

pub struct ComplexSpline {
    anchors: Vec<Anchor>,
    segments: Vec<Segment>,
}

type Critical = (Anchor, Segment);

impl ComplexSpline {
    pub fn insert(&mut self, (anchor, segment): Critical) -> usize {
        let index = self.anchors.quantified_insert(anchor);
        self.segments.insert(index, segment);
        index
    }

    //can't use index for the parallel vectors because that requires GATs
    //which at the time of writing this code is unstable
    pub fn anchor(&self, index: usize) -> &Anchor {
        &self.anchors[index]
    }

    pub fn anchor_mut(&mut self, index: usize) -> &mut Anchor {
        &mut self.anchors[index]
    }

    pub fn segment(&self, index: usize) -> &Segment {
        &self.segments[index]
    }

    pub fn segment_mut(&mut self, index: usize) -> &mut Segment {
        &mut self.segments[index]
    }

    pub fn remove(&mut self, index: usize) -> Critical {
        (self.anchors.remove(index), self.segments.remove(index))
    }

    pub fn replace(&mut self, index: usize, anch: Anchor) -> Anchor {
        self.anchors.quantified_replace(index, anch,
            |a, min, max| {
                let minx = min.unwrap_or(0.);
                let maxx = max.unwrap_or(f32::MAX);

                a.point.x = a.point.x.clamp(minx, maxx);
            }
        )
    }

    pub fn move_segment(&mut self, index: usize, pos: Point<f32>) -> Point<f32> {
        debug_assert!(index != 0, "can't move first segment");
        let old = self.segments[index].ctrls.get_end();
        self.segments[index].ctrls.set_end(pos);
        let end = self.segments[index - 1].ctrls.get_end();
        self.segments[index].recompute(end);
        old
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

        if old != new {
            *lutseeker = self.data[new].seeker();
        }

        lutseeker.method(t)
    }
}
//
//
//
//
//
/*#[cfg(test)]
mod tests {
    use super::*;
    use ggez::{
        event::{self, EventHandler, KeyCode, KeyMods, MouseButton},
        graphics::*,
        input::keyboard::is_key_pressed,
        timer::time_since_start,
        Context, GameResult,
    };
    use lyon_geom::Point;

    struct Test {
        cmpspl: ComplexSpline,
        point_buff: Vec<Point<f32>>,
        dimensions: Vec2,
        selection: Option<Critical>,
    }

    impl Test {
        fn new() -> GameResult<Self> {
            let x = 2000.;
            let y = 1000.;
            Ok(Self {
                cmpspl: ComplexSpline::new(x, Ctrl::Linear(Point::new(x, 0.))),
                point_buff: vec![],
                dimensions: Vec2::new(x, y),
                selection: None,
            })
        }
    }

    impl EventHandler for Test {
        fn update(&mut self, _ctx: &mut Context) -> GameResult {
            Ok(())
        }

        fn draw(&mut self, ctx: &mut Context) -> GameResult {
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

            let t =
                ((time_since_start(ctx).as_millis() as f32 % 5000.) / 5000.) * self.dimensions.x;

            let t_line = MeshBuilder::new()
                .polyline(
                    DrawMode::Stroke(StrokeOptions::DEFAULT),
                    &[
                        Vec2::new(0., self.dimensions.y * (3. / 4.)),
                        Vec2::new(0., self.dimensions.y),
                    ],
                    Color::WHITE,
                )?
                .build(ctx)?;

            draw(ctx, &t_line, (Vec2::new(t, 0.),))?;
            draw(ctx, &circle, (self.cmpspl.seeker().jump(t),))?;

            let mut seeker = self.cmpspl.automation.seeker();
            let res = 200;
            let auto_points: Vec<Vec2> = (0..res)
                .map(|x| {
                    Vec2::new(
                        (x as f32 / res as f32) * self.dimensions.x,
                        self.dimensions.y
                            - seeker.seek((x as f32 / res as f32) * self.dimensions.x)
                                * (self.dimensions.y / 4.),
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

            let rect = Mesh::new_rectangle(
                ctx,
                DrawMode::fill(),
                Rect::new(-5., -5., 10., 10.),
                Color::new(1., 0., 0., 0.5),
            )?;
            for i in 0..self.cmpspl.curve.segments().len() {
                let segment = &self.cmpspl.curve.segments()[i];
                match segment.ctrls {
                    Ctrl::Linear(p) => {
                        draw(ctx, &circle, (Vec2::new(p.x, p.y),))?;
                    }
                    Ctrl::Quadratic(p1, p2) => {
                        draw(ctx, &rect, (Vec2::new(p1.x, p1.y),))?;
                        draw(ctx, &circle, (Vec2::new(p2.x, p2.y),))?;
                    }
                    Ctrl::Cubic(p1, p2, p3) => {
                        let start = self.cmpspl.curve.segments()[i - 1].ctrls.get_end();
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

            for i in 1..self.cmpspl.curve.segments().len() {
                let mut points = Vec::<Vec2>::new();
                let mut seeker = self.cmpspl.curve.segments()[i].seeker();
                let mut t = 0.;
                while t <= 1. {
                    points.push(seeker.seek(t));
                    t += 0.05;
                }
                //let last = self.curve.segments[i].ctrls.end();
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
                    if self.dimensions.y * (3. / 4.) < y {
                        let c = self.cmpspl.closest_critical(x);
                        let dist = (self.cmpspl.get_critical_pos(c) - x).abs();
                        println!("nearest pont x distance to click: {}", dist);
                        if dist < 10. {
                            self.selection = Some(c);
                        } else {
                            match self.selection {
                                None => self.cmpspl.insert_critical(x),
                                Some(i) => {
                                    self.cmpspl.set_critical_pos(i, x);
                                    self.selection = None;
                                }
                            }
                        }
                    } else {
                        let c = self.cmpspl.closest_segment(Vec2::new(x, y));
                        if (self.cmpspl.curve[c.get()].ctrls.get_end() - Point::new(x, y)).length()
                            < 30.
                        {
                            self.selection = Some(c);
                        } else {
                            self.point_buff.push(Point::new(x, y));
                            println!("points: {:?}", self.point_buff);
                        }
                    }
                }
                MouseButton::Middle => {
                    if self.dimensions.y * (3. / 4.) < y {
                        let c = self.cmpspl.closest_critical(x);
                        self.cmpspl.cycle_weight(c);
                    }
                }
                _ => {}
            }
        }

        fn key_down_event(&mut self, _ctx: &mut Context, key: KeyCode, _mods: KeyMods, _: bool) {
            match key {
                KeyCode::Escape => {
                    self.selection = None;
                    return;
                }
                _ => {}
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
            self.point_buff.clear();

            if let Some(c) = self.selection {
                self.cmpspl.set_ctrls(c, ctrls);
                self.selection = None;
            }
        }

        fn mouse_wheel_event(&mut self, ctx: &mut Context, _x: f32, y: f32) {
            let pos = ggez::input::mouse::position(ctx);
            let c = if pos.y < self.dimensions.y * (3. / 4.) {
                self.cmpspl.closest_segment(pos.into())
            } else {
                self.cmpspl.closest_critical(pos.x)
            };
            if is_key_pressed(ctx, event::KeyCode::LShift) {
                self.cmpspl
                    .shift_period(c, if 0. < y { 10. } else { -10. })
                    .unwrap();
            } else {
                self.cmpspl
                    .shift_power(c, if 0. < y { 0.05 } else { -0.05 })
                    .unwrap();
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
