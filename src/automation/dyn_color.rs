use crate::{automation::*, utils::*};
use std::ops::Deref;
use ggez::graphics::Color as GColor;
use std::default::Default;

#[derive(Clone, Copy)]
struct Color(GColor);

impl Color {
    fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self(GColor {
            r, g, b, a
        })
    }
}

impl Deref for Color {
    type Target = GColor;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for Color {
    fn default() -> Self {
        Self(
            GColor::new(0., 0., 0., 0.)
        )
    }
}

impl BoundLerp for Color {
    fn blerp(self, other: &Self, amount: f32) -> Self {
        Color::new(
            (other.r - self.r) * amount + self.r,
            (other.g - self.g) * amount + self.g,
            (other.b - self.b) * amount + self.b,
            (other.a - self.a) * amount + self.a,
        )
    }
}

pub type DynColor = Automation<Color>;
pub type DynColorSeeker<'a> = AutomationSeeker<'a, Color>;
//
//
//
//
//
#[cfg(test)]
mod tests {
    use super::*;
    use ggez::{
        graphics::*,
        event::{self, EventHandler, MouseButton, KeyCode, KeyMods},
        timer::time_since_start,
        Context, GameResult, GameError,
    };
    use glam::Vec2;
    use tinyvec::tiny_vec;

    struct Test {
        color: DynColor,
        dimensions: Vec2,
    }

    impl Test {
        fn new() -> GameResult<Test> {
            let x: f32 = 2800.;
            Ok(Self {
                color: DynColor::new(
                    tiny_vec!([Epoch<ColorAnchor>; SHORT_ARR_SIZE] =>
                        (
                            0.,
                            ColorAnchor{
                                transition: Transition::Weighted(5.),
                                color: Color::BLACK
                            }
                        ).into(),
                        (
                            x / 2.,
                            ColorAnchor{
                                transition: Transition::Weighted(0.),
                                color: Color::new(1., 0., 0., 1.)
                            }
                        ).into()
                    ),
                    tiny_vec!([Epoch<ColorAnchor>; SHORT_ARR_SIZE] =>
                        (
                            0.,
                            ColorAnchor{
                                transition: Transition::Instant,
                                color: Color::WHITE,
                            }
                        ).into(),
                        (
                            x / 2.,
                            ColorAnchor{
                                transition: Transition::Weighted(0.),
                                color: Color::new(0., 1., 1., 1.),
                            }
                        ).into(),
                        (
                            x * (2. / 3.),
                            ColorAnchor{
                                transition: Transition::Instant,
                                color: Color::new(0., 1., 0., 1.)
                            }
                        ).into()
                    ),
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
                    col.val.color,
                )?;

                draw(
                    ctx,
                    &rect,
                    (Vec2::new(col.offset, self.dimensions.y - 20.),),
                )?;
            }

            for col in &self.color.upper {
                let rect = Mesh::new_rectangle(
                    ctx,
                    DrawMode::fill(),
                    Rect::new(0., 0., self.dimensions.x, 20.),
                    col.val.color,
                )?;

                draw(ctx, &rect, (Vec2::new(col.offset, 0.),))?;
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
