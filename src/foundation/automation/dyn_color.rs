use crate::{foundation::automation::*, utils::*};
use duplicate::duplicate;

pub enum Transition<T> {
    Instant(T),
    Lerp(T)
}

type Color = ggez::graphics::Color;
type ColorVecSeeker<'a> = BPSeeker<'a, Epoch<Transition<Color>>>;
impl<'a> Exhibit for ColorVecSeeker<'a> {
    fn exhibit(&self, _: f32) -> Color {
        let prev = self.previous();
    }
}

pub type DynColor = Automation<Vec<Epoch<Transition<Color>>>>;
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
                self.anchors.seeker(),
                self.lower.seeker(),
                self.upper.seeker(),
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
            Ok(Self {
                color: DynColor::new(
                    vec![
                        (0., Color::BLACK).into(),
                        (x / 2., Color::new(1., 0., 0., 1.)).into()
                    ],
                    vec![
                        (0., Color::WHITE).into(),
                        (x / 2., Color::new(0., 1., 1., 1.)).into(),
                        (x * (2. / 3.), Color::new(0., 1., 0., 1.)).into()
                    ],
                    x
                ),
                dimensions: Vec2::new(x, 1100.),
            })
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

            for col in &self.color.lower {
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

            for col in &self.color.upper {
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

            let mut seeker = self.color.anchors.seeker();
            let res = 2800;
            let points: Vec<Vec2> = (0..res)
                .map(|x| {
                    Vec2::new(
                        (x as f32 / res as f32) * self.dimensions.x,
                        self.dimensions.y 
                            - seeker.seek((x as f32 / res as f32) * self.dimensions.x)
                            * self.dimensions.y,
                    )
                })
                .collect();

            let lines = MeshBuilder::new()
                .polyline(
                    DrawMode::Stroke(StrokeOptions::DEFAULT),
                    points.as_slice(),
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
            let index = self.color.closest_to(ggez::input::mouse::position(ctx).into());
            match button {
                MouseButton::Left => {
                    self.color.insert(Anchor::new(Vec2::new(x, (self.dimensions.y - y) / self.dimensions.y)));
                }
                MouseButton::Middle => {
                    self.color[index].weight.cycle();
                }
                MouseButton::Right => {
                    self.color.remove(index);
                }
                _ => {}
            }
        }

        fn mouse_wheel_event(&mut self, ctx: &mut Context, _x: f32, y: f32) {
            let index = self
                .color
                .closest_to(ggez::input::mouse::position(ctx).into());
            let _ = self.color[index].weight.shift_curvature(
                if 0. < y { 0.05 } else { -0.05 }
            );
        }

        fn key_down_event(&mut self, ctx: &mut Context, key: KeyCode, _mods: KeyMods, _: bool) {
            let index = self.color
                .closest_to(ggez::input::mouse::position(ctx).into());
            automation::tests::key_handle(&mut self.color[index], key);
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
