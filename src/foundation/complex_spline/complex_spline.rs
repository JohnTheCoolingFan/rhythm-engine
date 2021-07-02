use crate::foundation::{automation::*, complex_spline::*};
use crate::utils::misc_traits::*;
use duplicate::{duplicate, duplicate_inline};
use glam::Vec2;
use lyon_geom::Point;

pub struct ComplexSpline {
    curve: CurveChain,
    automation: Automation,
}

#[derive(Debug, Copy, Clone)]
pub struct Critical(usize);

impl From<usize> for Critical {
    fn from(n: usize) -> Self {
        Critical((n + 1)/ 2)
    }
}

impl Into<usize> for Critical {
    fn into(self) -> usize {
        match self.0 {
            0 => 0,
            _ => self.0 * 2 - 1
        }
    }
}

impl Critical {
    pub fn get(&self) -> usize {
        self.0
    }
}

impl ComplexSpline {
    pub fn new(len: f32, intial: Ctrl) -> Self {
        let mut new_curve = CurveChain::new();
        new_curve.push_from_absolute(intial);
        let mut cmpspl = Self {
            curve: new_curve,
            automation: Automation::new(0., 1., len)
        };
        cmpspl.automation.set_pos(1, Vec2::new(len, 1.));

        cmpspl
    }
 
    pub fn bisect_segment(&mut self, c: Critical) {
        self.curve.bisect_segment(c.get());
        let start = self.automation.get_pos(c.into());
        let end = self.automation.get_pos(Critical(c.get() - 2).into());
        let x = end.x - start.x;

        self.automation.insert(Anchor::new(Vec2::new(x, 0.), Weight::ForwardBias));
        self.automation.insert(Anchor::new(Vec2::new(x, 1.), Weight::Curve(0.)));
    }

    pub fn set_ctrls(&mut self, index: Critical, ctrls: Ctrl) {
        self.curve.replace_from_absolute(index.0, ctrls)
    }

    pub fn get_weight(&mut self, c: Critical) -> Weight {
        self.automation.get_weight(c.into())
    }

    pub fn set_weight(&mut self, c: Critical, weight: Weight) {
        self.automation.set_weight(c.into(), weight)
    }

    pub fn insert_critical(&mut self, x: f32) {
        self.automation.insert(Anchor::new(Vec2::new(x, 0.), Weight::ForwardBias));
        self.automation.insert(Anchor::new(Vec2::new(x, 1.), Weight::Curve(0.)));

        let c: Critical = (self.automation.closest_to(Vec2::new(x, 0.))).into();
        self.curve.bisect_segment(c.get());
    }

    pub fn closest_segment(&self, point: Vec2) -> Critical {
        Critical(self.curve.closest_to(point))
    }

    pub fn closest_critical(&self, x: f32) -> Critical {
        self.automation.closest_to(Vec2::new(x, 0.)).into()
    }

    pub fn get_critical_pos(&mut self, c: Critical) -> f32 {
        self.automation.get_pos(c.into()).x
    }

    pub fn set_critical_pos(&mut self, c: Critical, x: f32) {
        debug_assert!(0 < c.get() && c.get() < Critical(self.automation.len()).into());
        let (ia, ya, ib, yb) = if self.automation.get_pos(c.into()).x < x {
            (Into::<usize>::into(c) + 1, 0., Into::<usize>::into(c), 1.)
        }
        else {
            (Into::<usize>::into(c), 1., Into::<usize>::into(c) + 1, 0.)
        };
        #[rustfmt::skip]
        duplicate_inline!{
            [
                i       y;
                [ia]    [ya];
                [ib]    [yb];
            ]
            self.automation.set_pos(i,
                Vec2::new(
                    x.clamp(
                        self.automation.get_pos(Critical(c.get() - 1).into()).x,
                        self.automation.get_pos(Critical(c.get() + 1).into()).x,
                    ),
                    y
                )
            );
        }
    }

    pub fn set_segment_pos(&mut self, index: usize, point: Vec2) {
        self.curve[index].ctrls.set_end(Point::new(point.x, point.y))
    }
}

pub struct CompSplSeeker<'a> {
    c_spline: &'a ComplexSpline,
    auto_seeker: AutomationSeeker<'a>,
    segment_seeker: SegmentSeeker<'a>,
}

impl<'a> Seeker<Vec2> for CompSplSeeker<'a> {
    #[duplicate(method; [seek]; [jump])]
    fn method(&mut self, val: f32) -> Vec2 {
        let old_index: Critical = self.auto_seeker.get_index().into();
        let y =  self.auto_seeker.method(val);
        let new_index: Critical = self.auto_seeker.get_index().into();
        if old_index.get() != new_index.get() {
            self.segment_seeker = self.c_spline.curve[new_index.get()].seeker();
        }
        self.segment_seeker.method(y)
    }
}

impl<'a> Seekable<'a> for ComplexSpline {
    type Output = Vec2;
    type SeekerType = CompSplSeeker<'a>;
    fn seeker(&'a self) -> Self::SeekerType {
        Self::SeekerType {
            auto_seeker: self.automation.seeker(),
            segment_seeker: self.curve[0].seeker(),
            c_spline: &self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ggez::{
        Context,
        GameResult,
        timer::time_since_start,
        event::{self, EventHandler, KeyCode, KeyMods, MouseButton},
        graphics::*
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
                    &[Vec2::new(0., self.dimensions.y * (3. / 4.)), Vec2::new(0., self.dimensions.y)],
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
                            * (self.dimensions.y / 4.)
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
                Color::new(1., 0., 0., 0.5)
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
            x: f32, y: f32
        ) {
            match button {
                MouseButton::Left => {
                    if self.dimensions.y * (3. / 4.) < y {
                        let c = self.cmpspl.closest_critical(x);
                        let dist = (self.cmpspl.get_critical_pos(c) - x).abs();
                        println!("nearest pont x distance to click: {}", dist);
                        if dist < 10. {
                            self.selection = Some(c);
                        }
                        else {
                            match self.selection {
                                None => { self.cmpspl.insert_critical(x) },
                                Some(i) => {
                                    self.cmpspl.set_critical_pos(i, x);
                                    self.selection = None;
                                }
                            }
                        }
                    }
                    else {
                        let c = self.cmpspl.closest_segment(Vec2::new(x, y));
                        if (self.cmpspl.curve[c.get()].ctrls.get_end() - Point::new(x, y)).length() < 30. {
                            self.selection = Some(c);
                        }
                        else {
                            self.point_buff.push(Point::new(x, y));
                            println!("points: {:?}", self.point_buff);
                        }
                    }
                }
                MouseButton::Middle => {
                    if self.dimensions.y * (3. / 4.) < y {
                        let c = self.cmpspl.closest_critical(x);
                        let weight = self.cmpspl.get_weight(c);

                        self.cmpspl.set_weight(
                        c,
                        match weight {
                            Weight::Curve(w) => {
                                if w != 0. {
                                    Weight::Curve(0.)
                                } else {
                                    Weight::ForwardBias
                                }
                            }
                            Weight::ForwardBias => Weight::ReverseBias,
                            Weight::ReverseBias => Weight::Curve(0.),
                        },
                    );

                        
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
            }
            else {
                self.cmpspl.closest_critical(pos.x)
            };
            let weight = self.cmpspl.get_weight(c);
            match weight {
                Weight::Curve(w) => self
                    .cmpspl
                    .set_weight(c, Weight::Curve(w + if 0. < y { 0.05 } else { -0.05 })),
                _ => {}
            };
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
