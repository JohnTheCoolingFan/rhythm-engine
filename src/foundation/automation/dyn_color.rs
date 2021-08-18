use crate::{foundation::automation::*, utils::*};
use duplicate::duplicate;

type Color = ggez::graphics::Color;

pub struct DynColor {
    pub automation: Automation,
    lower_colors: Vec<Epoch<Color>>,
    upper_colors: Vec<Epoch<Color>>, 
}

impl DynColor {
    pub fn new(len: f32) -> Self {
        Self {
            upper_colors: vec![(0., Color::WHITE).into()],
            automation: Automation::new(0., 1., len),
            lower_colors: vec![(0., Color::BLACK).into()],
        }
    }

    pub fn insert_upper(&mut self, color: Epoch<Color>) {
        self.upper_colors.quantified_insert(color);
    }

    pub fn insert_lower(&mut self, color: Epoch<Color>) {
        self.lower_colors.quantified_insert(color);
    }
}
//
//
//
//
//
type ColorVecSeeker<'a> = BPSeeker<'a, Epoch<Color>>;

impl<'a> Exhibit for ColorVecSeeker<'a> {
    fn exhibit(&self, _: f32) -> Color {
        self.previous().val
    }
}

type DynColSeekerMeta<'a> = (BPSeeker<'a, Anchor>, ColorVecSeeker<'a>, ColorVecSeeker<'a>);
pub type DynColorSeeker<'a> = Seeker<(), DynColSeekerMeta<'a>>;

impl<'a> SeekerTypes for DynColorSeeker<'a> {
    type Source = <BPSeeker<'a, Anchor> as SeekerTypes>::Source;
    type Output = Color;
}

impl<'a> Seek for DynColorSeeker<'a> {
    //Exhibit is immutable so have to do this
    #[duplicate(method; [seek]; [jump])]
    fn method(&mut self, offset: f32) -> Color {
        let (anchors, lower, upper) = &mut self.meta;
        let c1 = lower.method(offset);
        let t = anchors.method(offset);
        let c2 = upper.method(offset);
        Color::new(
            (c2.r - c1.r) * t + c1.r,
            (c2.g - c1.g) * t + c1.g,
            (c2.b - c1.b) * t + c1.b,
            (c2.a - c1.a) * t + c1.a,
        )
    }
}

impl<'a> Seekable<'a> for DynColor {
    type Seeker = DynColorSeeker<'a>;

    fn seeker(&'a self) -> Self::Seeker {
        Self::Seeker {
            data: (),
            meta: (
                self.automation.anchors.seeker(),
                self.lower_colors.seeker(),
                self.upper_colors.seeker(),
            )
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
    mod graficks {
        use ggez::graphics::Color;
        pub use ggez::graphics::*; //to remove colision
    }
    use super::*;
    use ggez::{
        event::{self, EventHandler, MouseButton, KeyCode, KeyMods},
        timer::time_since_start,
        Context, GameResult, GameError,
    };
    use glam::Vec2;
    use graficks::*;

    struct Test {
        color: DynColor,
        dimensions: Vec2,
    }

    impl Test {
        fn new() -> GameResult<Test> {
            let x: f32 = 2800.;
            let mut test = Self {
                color: DynColor::new(x),
                dimensions: Vec2::new(x, 1100.),
            };

            test.color
                .insert_lower((x / 2., Color::new(1., 0., 0., 1.)).into());
            test.color
                .insert_upper((x * (2. / 3.), Color::new(0., 1., 0., 1.)).into());
            test.color
                .insert_upper((x / 2., Color::new(0., 1., 1., 1.)).into());

            Ok(test)
        }
    }

    impl EventHandler<GameError> for Test {
        fn update(&mut self, _ctx: &mut Context) -> GameResult {
            Ok(())
        }

        fn draw(&mut self, ctx: &mut Context) -> GameResult {
            let t =
                ((time_since_start(ctx).as_millis() as f32 % 5000.) / 5000.) * self.dimensions.x;
            let mut seeker = self.color.seeker();
            clear(ctx, seeker.seek(t));

            for col in &self.color.lower_colors {
                let rect = Mesh::new_rectangle(
                    ctx,
                    DrawMode::fill(),
                    Rect::new(0., 0., self.dimensions.x, 20.),
                    col.val,
                )?;

                draw(
                    ctx,
                    &rect,
                    (Vec2::new(col.time, self.dimensions.y - 20.),),
                )?;
            }

            for col in &self.color.upper_colors {
                let rect = Mesh::new_rectangle(
                    ctx,
                    DrawMode::fill(),
                    Rect::new(0., 0., self.dimensions.x, 20.),
                    col.val,
                )?;

                draw(ctx, &rect, (Vec2::new(col.time, 0.),))?;
            }

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

            let mut auto_seeker = self.color.automation.seeker();
            let d = self.dimensions;
            let auto_points: Vec<Vec2> = (0..d.x as i32)
                .map(|x| {
                    Vec2::new(
                        (x as f32 / d.x) * d.x,
                        auto_seeker.seek((x as f32 / d.x) * d.x) * d.y,
                    )
                })
                .collect();

            let lines = MeshBuilder::new()
                .polyline(
                    DrawMode::Stroke(StrokeOptions::DEFAULT),
                    auto_points.as_slice(),
                    Color::new(1., 1., 1., 1.),
                )?
                .build(ctx)?;
            draw(ctx, &lines, (Vec2::new(0.0, 0.0),))?;

            let t_line = MeshBuilder::new()
                .polyline(
                    DrawMode::Stroke(StrokeOptions::DEFAULT),
                    &[Vec2::new(0., 0.), Vec2::new(0., self.dimensions.y)],
                    Color::WHITE,
                )?
                .build(ctx)?;
            draw(ctx, &t_line, (Vec2::new(t, 0.),))?;

            present(ctx)?;
            Ok(())
        }

        fn mouse_button_down_event(
            &mut self,
            ctx: &mut Context,
            button: MouseButton,
            x: f32,
            y: f32,
        ) {
            let automation = &mut self.color.automation;
            let index = automation.closest_to(ggez::input::mouse::position(ctx).into());
            match button {
                MouseButton::Left => {
                    automation.insert(Anchor::new(Vec2::new(x, (self.dimensions.y - y) / self.dimensions.y)));
                }
                MouseButton::Middle => {
                    automation[index].weight.cycle();
                }
                MouseButton::Right => {
                    automation.remove(index);
                }
                _ => {}
            }
        }

        fn mouse_wheel_event(&mut self, ctx: &mut Context, _x: f32, y: f32) {
            let index = self
                .color
                .automation
                .closest_to(ggez::input::mouse::position(ctx).into());
            if let Some(curvature) = self.color.automation[index].weight.curvature_mut() {
                    *curvature += if 0. < y { 0.05 } else { -0.05 };
                }
        }

        fn key_down_event(&mut self, ctx: &mut Context, key: KeyCode, _mods: KeyMods, _: bool) {
            let index = self.color
                .automation
                .closest_to(ggez::input::mouse::position(ctx).into());
            automation::tests::key_handle(&mut self.color.automation[index], key);
        }

    }

    #[test]
    pub fn dyncolor() -> GameResult {
        let state = Test::new()?;
        let cb = ggez::ContextBuilder::new("Dynamic Color test", "iiYese").window_mode(
            ggez::conf::WindowMode::default().dimensions(state.dimensions.x, state.dimensions.y),
        );
        let (ctx, event_loop) = cb.build()?;
        event::run(ctx, event_loop, state)
    }
}
