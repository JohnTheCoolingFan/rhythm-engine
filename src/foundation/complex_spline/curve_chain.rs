use crate::foundation::complex_spline::*;
use crate::utils::misc_traits::FromEnd;
use glam::Vec2;
use lyon_geom::Point;

pub struct CurveChain {
    segments: Vec<Segment>,
}

impl CurveChain {
    pub fn new() -> Self {
        Self {
            segments: vec![Segment::new(Ctrl::Linear(Point::new(0.0, 0.0)), 0.05)],
        }
    }

    pub fn push_from_absolute(&mut self, ctrls: Ctrl) {
        self.segments.push(Segment::new(
            match ctrls {
                Ctrl::Cubic(p0, p1, p2) => {
                    let start = self.segments[FromEnd(0)].ctrls.end();
                    let a1 = Point::new(p0.x - start.x, p0.y - start.y);
                    let a2 = p1.to_vector() - p2.to_vector();
                    Ctrl::Cubic(a1, Point::new(a2.x, a2.y), p2)
                }
                _ => ctrls,
            },
            0.05,
        ));
        let p = self.segments[FromEnd(1)].ctrls.end();
        self.segments[FromEnd(0)].recompute(p);
    }

    pub fn pop(&mut self) -> Segment {
        self.segments.pop().unwrap()
    }

    pub fn replace_from_absolute(&mut self, index: usize, ctrls: Ctrl) {
        debug_assert!(0 < index && index < self.segments.len());
        self.segments[index].ctrls = match ctrls {
            Ctrl::Cubic(p0, p1, p2) => {
                let start = self.segments[index - 1].ctrls.end();
                let a1 = Point::new(p0.x - start.x, p0.y - start.y);
                let a2 = p1.to_vector() - p2.to_vector();
                Ctrl::Cubic(a1, Point::new(a2.x, a2.y), p2)
            }
            _ => ctrls,
        };
        let p0 = self.segments[index - 1].ctrls.end();
        self.segments[index].recompute(p0);
        if index + 1 < self.segments.len() {
            self.segments[index + 1].recompute(ctrls.end());
        }
    }

    pub fn bisect_segment(&mut self, index: usize) {
        debug_assert!(0 < index && index < self.segments.len());
        let start = self.segments[index - 1].ctrls.end();
        let end = self.segments[index].ctrls.end();
        self.segments[index].ctrls =
            Ctrl::Linear(start + ((end.to_vector() - start.to_vector()) * (1. / 2.)));
        self.segments
            .insert(index + 1, Segment::new(Ctrl::Linear(end), 0.5));

        let p0 = self.segments[index - 1].ctrls.end();
        let p1 = self.segments[index].ctrls.end();
        self.segments[index].recompute(p0);
        self.segments[index + 1].recompute(p1);
    }

    pub fn remove(&mut self, index: usize) -> Segment {
        debug_assert!(0 < index && index < self.segments.len());
        let segment = self.segments.remove(index);
        let p0 = self.segments[index - 1].ctrls.end();
        self.segments[index].recompute(p0);
        segment

    }

    pub fn clear(&mut self) {
        self.segments.clear();
        self.segments
            .push(Segment::new(Ctrl::Linear(Point::new(0.0, 0.0)), 0.05));
    }

    pub fn closest_to(&self, point: Vec2) -> usize {
        let (index, _) = self
            .segments
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                let p = a.ctrls.end();
                let q = b.ctrls.end();
                (Vec2::new(p.x, p.y) - point)
                    .length()
                    .partial_cmp(&(Vec2::new(q.x, q.y) - point.into()).length())
                    .unwrap()
            })
            .unwrap();

        index
    }
}

impl std::ops::Index<usize> for CurveChain {
    type Output = Segment;
    fn index(&self, n: usize) -> &Segment {
        &self.segments[n]
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::{Seekable, Seeker};

    use super::*;
    use ggez::graphics::*;
    use ggez::{
        event::{self, EventHandler, KeyCode, KeyMods, MouseButton},
        input::keyboard::is_key_pressed,
        graphics::MeshBuilder,
    };
    use ggez::{Context, GameResult};
    use glam::Vec2;

    struct Test {
        curve: CurveChain,
        point_buff: Vec<Point<f32>>,
        selected_segment: Option<usize>,
    }

    impl Test {
        fn new() -> GameResult<Test> {
            Ok(Test {
                curve: CurveChain::new(),
                point_buff: vec![],
                selected_segment: None,
            })
        }
    }

    impl EventHandler for Test {
        fn update(&mut self, _ctx: &mut Context) -> GameResult {
            Ok(())
        }

        fn draw(&mut self, ctx: &mut Context) -> GameResult {
            clear(ctx, Color::new(0., 0., 0., 1.));
            let mouse_pos = ggez::input::mouse::position(ctx);

            let circle = Mesh::new_circle(
                ctx,
                DrawMode::fill(),
                Vec2::new(0.0, 0.0),
                10.0,
                2.0,
                Color::new(1.0, 1.0, 1.0, 1.0),
            )?;
            draw(ctx, &circle, (Vec2::new(mouse_pos.x, mouse_pos.y),))?;

            for i in 1..self.curve.segments.len() {
                let segment = &self.curve.segments[i];
                match segment.ctrls {
                    Ctrl::Linear(p) => {
                        draw(ctx, &circle, (Vec2::new(p.x, p.y),))?;
                    }
                    Ctrl::Quadratic(p1, p2) => {
                        draw(ctx, &circle, (Vec2::new(p1.x, p1.y),))?;
                        draw(ctx, &circle, (Vec2::new(p2.x, p2.y),))?;
                    }
                    Ctrl::Cubic(p1, p2, p3) => {
                        let start = self.curve.segments[i - 1].ctrls.end();
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

            for i in 1..self.curve.segments.len() {
                let mut points = Vec::<Vec2>::new();
                let mut seeker = self.curve.segments[i].seeker();
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

        fn key_down_event(&mut self, _ctx: &mut Context, key: KeyCode, _mods: KeyMods, _: bool) {
            match key {
                KeyCode::Escape => {
                    self.selected_segment = None;
                    return;
                }
                KeyCode::C => {
                    self.curve.clear();
                }
                KeyCode::Space => {
                    self.selected_segment.map(|index| self.curve.bisect_segment(index));
                    self.selected_segment = None;
                }
                KeyCode::Delete => {
                    self.selected_segment.map(|index| self.curve.remove(index));
                    self.selected_segment = None;
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

            match self.selected_segment {
                None => {
                    self.curve.push_from_absolute(ctrls);
                }
                Some(index) => {
                    self.curve.replace_from_absolute(index, ctrls);
                    self.selected_segment = None;
                }
            }
        }

        fn mouse_button_down_event(
            &mut self,
            ctx: &mut Context,
            button: MouseButton,
            x: f32,
            y: f32,
        ) {
            match button {
                MouseButton::Left => {
                    if is_key_pressed(ctx, KeyCode::LShift) {
                        self.selected_segment = Some(self.curve.closest_to(Vec2::new(x, y)));
                    }
                    else {
                        println!("click");
                        self.point_buff.push(Point::new(x, y));
                        println!("{:?}", self.point_buff);
                    }
                }
                _ => {}
            }
        }
    }

    #[test]
    pub fn curves() -> GameResult {
        let cb = ggez::ContextBuilder::new("Curve test", "iiYese")
            .window_mode(ggez::conf::WindowMode::default().dimensions(1920., 1080.));
        let (ctx, event_loop) = cb.build()?;
        let state = Test::new()?;
        event::run(ctx, event_loop, state)
    }
}
