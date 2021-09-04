use crate::{automation::*, utils::*};
use duplicate::duplicate;
use ggez::graphics::Color;
use tinyvec::TinyVec;
use std::default::Default;

#[derive(Clone, Copy)]
pub enum Transition {
    Instant,
    Weighted(f32)
}

impl Transition {
    pub fn cycle(&mut self) {
        *self = match self {
            Self::Instant => Self::Weighted(0.),
            Self::Weighted(_) => Self::Instant
        }
    }
}

#[derive(Clone, Copy)]
pub struct ColorAnchor {
    pub color: Color,
    pub transition: Transition
}

impl Default for ColorAnchor {
    fn default() -> Self {
        Self {
            color: Color::new(0., 0., 0., 0.),
            transition: Transition::Instant
        }
    }
}

type ColorVecSeeker<'a> = Seeker<&'a TinyVec<[Epoch<ColorAnchor>; 3]>, usize>;
impl<'a> Exhibit for ColorVecSeeker<'a> {
    //must return Anchor because of the way Epoch and Exhibit are implemented
    fn exhibit(&self, offset: f32) -> ColorAnchor {
        match (self.previous(), self.current()) {
            (_, Err(curr)) | (None, Ok(curr) | Err(curr)) => curr.val,
            (Some(prev), Ok(curr)) => {
                match prev.val.transition {
                    Transition::Instant => prev.val,
                    Transition::Weighted(weight) => {
                        let t = (offset - prev.time) / (curr.time - prev.time);
                        let w = Weight::QuadLike{ 
                            curvature: weight,
                            x_flip: false,
                            y_flip: false
                        }.eval(t);
                        let (c1, c2) = (prev.val.color, curr.val.color);
                        ColorAnchor{
                            color: Color::new(
                                (c2.r - c1.r) * w + c1.r,
                                (c2.g - c1.g) * w + c1.g,
                                (c2.b - c1.b) * w + c1.b,
                                (c2.a - c1.a) * w + c1.a,
                            ),
                            .. prev.val
                        }
                    }
                }
            }
        }
    }
}

pub type DynColor = Automation<Vec<Epoch<ColorAnchor>>>;
type DynColSeekerMeta<'a> = (Seeker<&'a TinyVec<[Anchor; 3]>, usize>, ColorVecSeeker<'a>, ColorVecSeeker<'a>);
pub type DynColorSeeker<'a> = Seeker<(), DynColSeekerMeta<'a>>;

impl<'a> SeekerTypes for DynColorSeeker<'a> {
    type Source = <Seeker<&'a TinyVec<[Anchor; 3]>, usize> as SeekerTypes>::Source;
    type Output = Color;
}

impl<'a> Seek for DynColorSeeker<'a> {
    //Exhibit is immutable so have to do this
    #[duplicate(method; [seek]; [jump])]
    fn method(&mut self, offset: f32) -> Color {
        let (anchors, lower, upper) = &mut self.meta;
        let c1 = lower.method(offset).color;
        let t = anchors.method(offset);
        let c2 = upper.method(offset).color;
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
    use super::*;
    use ggez::{
        graphics::*,
        event::{self, EventHandler, MouseButton, KeyCode, KeyMods},
        timer::time_since_start,
        Context, GameResult, GameError,
    };
    use glam::Vec2;

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
                    ],
                    vec![
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
                    col.val.color,
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
                    col.val.color,
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
