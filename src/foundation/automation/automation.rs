use crate::foundation::automation::anchor::*;
use crate::utils::seeker::*;
use glam::Vec2;
use std::ops::{Index, IndexMut};
use duplicate::duplicate;

pub struct Automation {
    pub upper_bound: f32,
    pub lower_bound: f32,
    anchors: Vec<Anchor>,
}

impl Index<usize> for Automation {
    type Output = Anchor;

    fn index(&self, n: usize) -> &Self::Output {
        &self.anchors[n]
    }
}

impl IndexMut<usize> for Automation {
    fn index_mut(&mut self, n: usize) -> &mut Anchor {
        &mut self.anchors[n]
    }
}

impl Automation {
    pub fn new(lb: f32, ub: f32, len: f32) -> Self {
        assert!(lb < ub, "upper bound must be greater than lower bound");
        assert!(0. < len, "length cannot be zero or negative");
        Automation {
            upper_bound: ub,
            lower_bound: lb,
            anchors: vec![
                Anchor::new(Vec2::new(0., 0.0)),
                Anchor::new(Vec2::new(len, 0.0)),
            ],
        }
    }

    pub fn len(&self) -> usize {
        self.anchors.len()
    }

    pub fn insert(&mut self, anch: Anchor) -> usize {
        self.anchors.quantified_insert(anch)
    }

    pub fn remove(&mut self, index: usize) -> Anchor {
        self.anchors.remove(index)
    }

    pub fn closest_to(&self, point: Vec2) -> usize {
        let (index, _) = self
            .anchors
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                (*a.point() - point)
                    .length()
                    .partial_cmp(&(*b.point() - point).length())
                    .unwrap()
            })
            .unwrap();

        index
    }

    pub fn set_pos(&mut self, index: usize, mut point: Vec2) -> Vec2 {
        debug_assert!(index < self.anchors.len(), "out of bounds index");
        let old = *self.anchors[index].point();
        let minx = if index == 0 {
            0.
        } else {
            self.anchors[index - 1].point().x
        };
        let maxx = if self.anchors.len() - index <= 1 {
            f32::MAX
        } else {
            self.anchors[index + 1].point.x
        };

        point.x = point.x.clamp(minx, maxx);
        point.y = point.y.clamp(0., 1.);
        self.anchors[index].point = point;
        old
    }
}
//
//
//
//
//
type AnchVecSeeker<'a> = <Vec<Anchor> as Seekable<'a>>::Seeker;
type AutomationSeeker<'a> = Seeker<(f32, f32), AnchVecSeeker<'a>>;

impl<'a> SeekerTypes for AutomationSeeker<'a> {
    type Source = Anchor;
    type Output = f32;
}

impl<'a> Seek for AutomationSeeker<'a> {
    #[duplicate(method; [seek]; [jump])]
    fn method(&mut self, offset: f32) -> f32 {
        let (lb, ub) = self.data;
        lb + (ub - lb) * self.meta.method(offset)
    }
}

impl<'a> Seekable<'a> for Automation {
    type Seeker = AutomationSeeker<'a>;

    fn seeker(&'a self) -> Self::Seeker {
        Self::Seeker {
            meta: self.anchors.seeker(),
            data: (self.upper_bound, self.lower_bound)
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
        event::{self, EventHandler, MouseButton, KeyCode, KeyMods},
        graphics::*,
        input::keyboard::is_key_pressed,
    };
    use ggez::{Context, GameResult, GameError};

    struct Test {
        automation: Automation,
        dimensions: Vec2,
    }

    impl Test {
        fn new() -> GameResult<Self> {
            Ok(Self {
                automation: Automation::new(0., 1., 2800.),
                dimensions: Vec2::new(2800., 1100.),
            })
        }
    }

    impl EventHandler<GameError> for Test {
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

            let mut seeker = self.automation.seeker();
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
            let index = self
                .automation
                .closest_to(ggez::input::mouse::position(ctx).into());
            match button {
                MouseButton::Left => {
                    self.automation
                        .insert(Anchor::new(Vec2::new(x, y / self.dimensions.y)));
                }
                MouseButton::Middle => {
                    self.automation[index].weight.cycle();
                }
                MouseButton::Right => {
                    self.automation.remove(index);
                }
                _ => {}
            }
        }

        fn mouse_wheel_event(&mut self, ctx: &mut Context, _x: f32, y: f32) {
            let index = self
                .automation
                .closest_to(ggez::input::mouse::position(ctx).into());
            let _ = self.automation[index]
                .weight
                .shift_power(if 0. < y { 0.05 } else { -0.05 });
        }

        fn key_down_event(&mut self, ctx: &mut Context, key: KeyCode, mods: KeyMods, _: bool) {
            let index = self
                .automation
                .closest_to(ggez::input::mouse::position(ctx).into());
            match key {
                KeyCode::Q => {
                    let _ =self.automation[index].subwave.mode.toggle_x_alternate();
                }
                KeyCode::E => {
                    let _ = self.automation[index].subwave.mode.toggle_y_alternate();
                }
                KeyCode::D => {
                    self.automation[index].subwave.shift_period(2.);
                }
                KeyCode::A => {
                    self.automation[index].subwave.shift_period(-2.);
                }
                KeyCode::W => {
                    let _ = self.automation[index].subwave.weight.shift_power(2.);
                }
                KeyCode::S => {
                    let _ = self.automation[index].subwave.weight.shift_power(-2.);
                }
                KeyCode::Key1 => {
                    self.automation[index].subwave.offset -= 2.;
                }
                KeyCode::Key2 => {
                    self.automation[index].subwave.offset += 2.;
                }
                KeyCode::Key3 => {
                    self.automation[index].subwave.weight.cycle();
                }
                KeyCode::Key4 => {
                    self.automation[index].subwave.mode.cycle();
                }
                _ => (),
            }
        }
    }

    #[test]
    pub fn automation() -> GameResult {
        let state = Test::new()?;
        let cb = ggez::ContextBuilder::new("Automation test", "iiYese").window_mode(
            ggez::conf::WindowMode::default().dimensions(state.dimensions.x, state.dimensions.y),
        );
        let (ctx, event_loop) = cb.build()?;
        event::run(ctx, event_loop, state)
    }
}
