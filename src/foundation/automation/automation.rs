use crate::foundation::automation::anchor::*;
use crate::utils::seeker::*;
use glam::Vec2;
use std::ops::{Index, IndexMut};
use duplicate::duplicate;

pub struct Automation<T> {
    pub upper: T,
    pub lower: T,
    pub(super) anchors: Vec<Anchor>,
}

impl<T> Index<usize> for Automation<T> {
    type Output = Anchor;

    fn index(&self, n: usize) -> &Self::Output {
        &self.anchors[n]
    }
}

impl<T> IndexMut<usize> for Automation<T> {
    fn index_mut(&mut self, n: usize) -> &mut Anchor {
        &mut self.anchors[n]
    }
}

impl<T> Automation<T> {
    pub fn new(lower: T, upper: T, len: f32) -> Self {
        Self {
            upper,
            lower,
            anchors: vec![
                Anchor::new(Vec2::new(0., 0.0)),
                Anchor::new(Vec2::new(len, 0.0)),
            ],
        }
    }

    fn correct_anchor(&mut self, index: usize) {
        assert!((1..self.anchors.len()).contains(&index));
        self.anchors[index].point.x = self.anchors[index].point.x.clamp(
            self.anchors[index - 1].point.x,
            if index + 1 < self.anchors.len() {
                self.anchors[index + 1].point.x
            }
            else {
                f32::MAX
            }
        );
    }


    pub fn len(&self) -> usize {
        self.anchors.len()
    }

    pub fn insert(&mut self, anch: Anchor) -> usize {
        let index = self.anchors.quantified_insert(anch);
        self.correct_anchor(index);
        index
    }

    pub fn remove(&mut self, index: usize) -> Anchor {
        self.anchors.remove(index)
    }

    pub fn closest_to(&self, point: Vec2) -> usize {
        self.anchors
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                (a.point - point)
                    .length()
                    .partial_cmp(&(b.point - point).length())
                    .unwrap()
            })
            .unwrap().0
    }
}
//
//
//
//
//

type AnchVecSeeker<'a> = BPSeeker<'a, Anchor>;
pub type AutomationSeeker<'a, T> = Seeker<(T, T), AnchVecSeeker<'a>>;

impl<'a> SeekerTypes for AutomationSeeker<'a, f32> {
    type Source = Anchor;
    type Output = f32;
}

impl<'a> Seek for AutomationSeeker<'a, f32> {
    #[duplicate(method; [seek]; [jump])]
    fn method(&mut self, offset: f32) -> f32 {
        let (lb, ub) = self.data;
        lb + (ub - lb) * self.meta.method(offset)
    }
}

impl<'a> Seekable<'a> for Automation<f32> {
    type Seeker = AutomationSeeker<'a, f32>;

    fn seeker(&'a self) -> Self::Seeker {
        Self::Seeker {
            meta: self.anchors.seeker(),
            data: (self.upper, self.lower)
        }
    }
}
//
//
//
//
//
#[cfg(test)]
pub mod tests {
    use super::*;
    use ggez::{
        event::{self, EventHandler, MouseButton, KeyCode, KeyMods},
        graphics::*,
    };
    use ggez::{Context, GameResult, GameError};

    struct Test {
        automation: Automation<f32>,
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

    pub fn key_handle(anch: &mut Anchor, key: KeyCode) {
        match key {
            KeyCode::Q => {
                if let Some(flip) = anch.subwave.weight.x_flip_mut() {
                    *flip = !*flip;
                }
            }
            KeyCode::E => {
                let flip = anch.subwave.weight.y_flip_mut();
                *flip = !*flip;
            }
            KeyCode::F => {
                if let Some(flip) = anch.weight.x_flip_mut() {
                    *flip = !*flip;
                }
            }
            KeyCode::C => {
                let flip = anch.weight.y_flip_mut();
                *flip = !*flip;
            }
            KeyCode::D => {
                anch.subwave.shift_period(2.);
            }
            KeyCode::A => {
                anch.subwave.shift_period(-2.);
            }
            KeyCode::W => {
                let _ = anch.subwave.weight.shift_curvature(0.05);
            }
            KeyCode::S => {
                let _ = anch.subwave.weight.shift_curvature(-0.05);
            }
            KeyCode::Key1 => {
                anch.subwave.offset -= 2.;
            }
            KeyCode::Key2 => {
                anch.subwave.offset += 2.;
            }
            KeyCode::Key3 => {
                anch.subwave.weight.cycle();
            }
            KeyCode::Key4 => {
                anch.subwave.mode.cycle();
            }
            _ => (),
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
            let _ = self.automation[index].weight.shift_curvature(
                if 0. < y { 0.05 } else { -0.05 }
            );
        }

        fn key_down_event(&mut self, ctx: &mut Context, key: KeyCode, _mods: KeyMods, _: bool) {
            let index = self.automation.closest_to(ggez::input::mouse::position(ctx).into());
            key_handle(
                &mut self.automation[index],
                key
            );
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
