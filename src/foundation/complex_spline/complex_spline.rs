use crate::foundation::{automation::*, complex_spline::*};
use crate::utils::*;
use duplicate::duplicate;
use glam::Vec2;

use super::segment::SegmentSeeker;

struct ComplexSpline {
    curve: CurveChain,
    offset: f32,
    automation: Automation,
}

impl ComplexSpline {
    pub fn new(start: f32, len: f32, intial: Ctrl) -> Self {
        let mut new_curve = CurveChain::new();
        new_curve.push_from_absolute(intial);
        Self {
            curve: new_curve,
            offset: start,
            automation: Automation::new(0., 1., len, false),
        }
    }
}

struct CompSplSeeker<'a> {
    c_spline: &'a ComplexSpline,
    auto_seeker: AutomationSeeker<'a>,
    segment_seeker: SegmentSeeker<'a>,
}

impl<'a> Seeker<Vec2> for CompSplSeeker<'a> {
    #[duplicate(method; [seek]; [jump])]
    fn method(&mut self, val: f32) -> Vec2 {
        let old_index = self.auto_seeker.get_index();
        let y = self.auto_seeker.method(val - self.c_spline.offset);
        let new_index = self.auto_seeker.get_index();
        if old_index != new_index {
            self.segment_seeker = self.c_spline.curve[new_index].seeker();
        }
        self.segment_seeker.method(y)
    }
}

impl<'a> Seekable<'a, Vec2> for ComplexSpline {
    type Seeker = CompSplSeeker<'a>;
    fn seeker(&'a self) -> Self::Seeker {
        CompSplSeeker {
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
        event::{self, EventHandler, KeyCode, KeyMods, MouseButton},
        graphics::*,
        Context, GameResult,
    };
    use lyon_geom::Point;

    struct Test {
        cps: ComplexSpline,
        dimensions: Vec2,
    }

    impl Test {
        fn new() -> GameResult<Self> {
            let x = 2800.;
            let y = 1000.;
            Ok(Self {
                cps: ComplexSpline::new(0., x, Ctrl::Linear(Point::new(x, 0.))),
                dimensions: Vec2::new(x, y),
            })
        }
    }

    impl EventHandler for Test {
        fn update(&mut self, _ctx: &mut Context) -> GameResult {
            Ok(())
        }

        fn draw(&mut self, ctx: &mut Context) -> GameResult {
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

            present(ctx)?;
            Ok(())
        }
    }

    #[cfg(test)]
    fn complex_spline() -> GameResult {
        let state = Test::new()?;
        let cb = ggez::ContextBuilder::new("Complex Spline test", "iiYese").window_mode(
            ggez::conf::WindowMode::default().dimensions(state.dimensions.x, state.dimensions.y),
        );
        let (ctx, event_loop) = cb.build()?;
        event::run(ctx, event_loop, state)
    }
}
