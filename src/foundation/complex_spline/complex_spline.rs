use crate::foundation::{automation::*, complex_spline::*};
use crate::utils::misc_traits::*;
use duplicate::{duplicate, duplicate_inline};
use glam::Vec2;
use lyon_geom::Point;

pub struct ComplexSpline {
    curve: CurveChain,
    automation: Automation,
}

//automation to critical index
fn atoc_index(index: usize) -> usize {
    (index + 1) / 2 
}

//critical to automation index
fn ctoa_index(index: usize) -> usize {
    match index {
        0 => 0,
        _ => index * 2 - 1
    }
}

impl ComplexSpline {
    pub fn new(len: f32, intial: Ctrl) -> Self {
        let mut new_curve = CurveChain::new();
        new_curve.push_from_absolute(intial);
        let mut cmpspl = Self {
            curve: new_curve,
            automation: Automation::new(0., 1., len, false)
        };
        cmpspl.automation.set_pos(1, Vec2::new(len, 1.));

        cmpspl
    }

    pub fn curve(&self) -> &CurveChain {
        &self.curve
    }

    pub fn automation(&self) -> &Automation {
        &self.automation
    }

    pub fn bisect_segment(&mut self, index: usize) {
        self.curve.bisect_segment(index);
        let start = self.automation.get_pos(ctoa_index(index));
        let end = self.automation.get_pos(ctoa_index(index - 2));
        let x = end.x - start.x;

        self.automation.insert(Anchor::new(Vec2::new(x, 0.), Weight::ForwardBias));
        self.automation.insert(Anchor::new(Vec2::new(x, 1.), Weight::Curve(0.)));
    }

    pub fn insert_critical(&mut self, x: f32) {
        self.automation.insert(Anchor::new(Vec2::new(x, 0.), Weight::ForwardBias));
        self.automation.insert(Anchor::new(Vec2::new(x, 1.), Weight::Curve(0.)));

        let index = atoc_index(self.automation.closest_to(Vec2::new(x, 0.)));
        self.curve.bisect_segment(index);
    }

    pub fn closest_segment(&self, point: Vec2) -> usize {
        self.curve.closest_to(point)
    }

    pub fn closest_critical(&self, x: f32) -> usize {
        atoc_index(self.automation.closest_to(Vec2::new(x, 0.)))
    }

    pub fn set_critical_pos(&mut self, index: usize, x: f32) {
        debug_assert!(0 < index && index < atoc_index(self.automation.len()));
        duplicate_inline!{
            [
                i;
                [ctoa_index(index)];
                [ctoa_index(index) - 1]
            ]
            self.automation.set_pos(i,
                Vec2::new(
                    x.clamp(
                        self.automation.get_pos(ctoa_index(index - 1)).x,
                        self.automation.get_pos(ctoa_index(index + 1)).x,
                    ),
                    self.automation.get_pos(ctoa_index(index)).y
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
        let old_index = atoc_index(self.auto_seeker.get_index());
        let y =  self.auto_seeker.method(val);
        let new_index = atoc_index(self.auto_seeker.get_index());
        if old_index != new_index {
            self.segment_seeker = self.c_spline.curve[atoc_index(new_index)].seeker();
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
    use ggez::{Context, GameResult, event::{self, EventHandler, KeyCode, KeyMods, MouseButton}, graphics::*, input::keyboard::{is_key_pressed, is_key_repeated}};
    use lyon_geom::Point;

    struct Test {
        cmpspl: ComplexSpline,
        point_buff: Vec<Point<f32>>,
        dimensions: Vec2,
        selection: Option<usize>,
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

            for i in 0..self.cmpspl.curve.segments().len() {
                let segment = &self.cmpspl.curve.segments()[i];
                match segment.ctrls {
                    Ctrl::Linear(p) => {
                        draw(ctx, &circle, (Vec2::new(p.x, p.y),))?;
                    }
                    Ctrl::Quadratic(p1, p2) => {
                        draw(ctx, &circle, (Vec2::new(p1.x, p1.y),))?;
                        draw(ctx, &circle, (Vec2::new(p2.x, p2.y),))?;
                    }
                    Ctrl::Cubic(p1, p2, p3) => {
                        let start = self.cmpspl.curve.segments()[i - 1].ctrls.get_end();
                        draw(ctx, &circle, (Vec2::new(start.x + p1.x, start.y + p1.y),))?;
                        draw(ctx, &circle, (Vec2::new(p2.x + p3.x, p2.y + p3.y),))?;
                        draw(ctx, &circle, (Vec2::new(p3.x, p3.y),))?;
                    }
                    Ctrl::ThreePointCircle(p1, p2) => {
                        draw(ctx, &circle, (Vec2::new(p1.x, p1.y),))?;
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
                        let index = self.cmpspl.closest_critical(x);
                        if self.cmpspl.automation().get_pos(index).x - x < 5. {
                            self.selection = Some(atoc_index(index));
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
                        let index = self.cmpspl.curve().closest_to(Vec2::new(x, y));
                        if (self.cmpspl.curve()[ctoa_index(index)].ctrls.get_end() - Point::new(x, y)).length() < 5. {
                            self.selection = Some(index);
                        }
                        else {
                            match self.selection {
                                None => { self.point_buff.push(Point::new(x, y)) },
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
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
        println!("gothre");
        event::run(ctx, event_loop, state)
    }
}
