use super::anchor::*;
use crate::utils::*;
use glam::Vec2;
use std::ops::{Index, IndexMut};
use duplicate::duplicate;
use tinyvec::tiny_vec;

pub struct Automation<T>
where
    T: Default + BoundLerp
{
    pub upper: TVec<Epoch<TransitionedBound<T>>>,
    pub lower: TVec<Epoch<TransitionedBound<T>>>,
    pub(super) anchors: TVec<Anchor>,
}

impl<T> Index<usize> for Automation<T>
where
    T: Default + BoundLerp
{
    type Output = Anchor;

    fn index(&self, n: usize) -> &Self::Output {
        &self.anchors[n]
    }
}

impl<T> IndexMut<usize> for Automation<T> 
where
    T: Default + BoundLerp
{
    fn index_mut(&mut self, n: usize) -> &mut Anchor {
        &mut self.anchors[n]
    }
}

impl<T> Automation<T>
where
    T: Default + BoundLerp
{
    pub fn new(initial_lower: T, initial_upper: T, len: f32) -> Self {
        Self {
            upper: tiny_vec!([Epoch<TransitionedBound<T>>; SHORT_ARR_SIZE] =>
                Epoch::<TransitionedBound<T>>{ offset: 0., val: TransitionedBound::<T>{
                        transition: Transition::Instant,
                        val: initial_upper
                    }
                }
            ),
            lower: tiny_vec!([Epoch<TransitionedBound<T>>; SHORT_ARR_SIZE] =>
                Epoch::<TransitionedBound<T>>{ offset: 0., val: TransitionedBound::<T>{
                        transition: Transition::Instant,
                        val: initial_lower
                    }
                }
            ),
            anchors: tiny_vec!([Anchor; SHORT_ARR_SIZE] =>
                Anchor::new(Vec2::new(0., 0.0)),
                Anchor::new(Vec2::new(len, 0.0))
            ),
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

impl Default for Transition {
    fn default() -> Self {
        Self::Instant
    }
}

#[derive(Default, Clone, Copy)]
pub struct TransitionedBound<T>
where
    T: BoundLerp + Default
{
    pub transition: Transition,
    pub val: T
}

pub trait BoundLerp {
    fn blerp(self, other: &Self, amount: f32) -> Self;
}

//Transitionable Bounds Vector
pub type TBVSeeker<'a, T> = Seeker<&'a TVec<Epoch<TransitionedBound<T>>>, usize>;
impl<'a, T> Exhibit for TBVSeeker<'a, T>
where
    T: BoundLerp + Default + Copy
{
    fn exhibit(&self, offset: f32) -> TransitionedBound<T> {
        match (self.previous(), self.current()) {
            (None, Ok(curr) | Err(curr)) | (_, Err(curr)) => curr.val,
            (Some(prev), Ok(curr)) => {
                match prev.val.transition {
                    Transition::Instant => prev.val,
                    Transition::Weighted(weight) => {
                        TransitionedBound::<T> {
                            //mmm yes naming
                            val: prev.val.val.blerp(&curr.val.val,
                                Weight::QuadLike{ 
                                    curvature: weight,
                                    x_flip: false,
                                    y_flip: false
                                }.eval(
                                    (offset - prev.offset) / (curr.offset - prev.offset)
                                )
                            ),
                            .. prev.val
                        }
                    }
                }
            }
        }
    }
}
//
//
//
//
//
pub type AutomationSeeker<'a, T> = Seeker<
    (),
    (TBVSeeker<'a, T>, Seeker<&'a TVec<Anchor>, usize>, TBVSeeker<'a, T>)
>;

impl<'a, T> SeekerTypes for AutomationSeeker<'a, T> 
where
    T: BoundLerp + Copy + Default
{
    type Source = Anchor;
    type Output = T;
}

impl<'a, T> Seek for AutomationSeeker<'a, T>
where
    T: BoundLerp + Copy + Default
{
    #[duplicate(method; [seek]; [jump])]
    fn method(&mut self, offset: f32) -> T {
        let (ref mut lower_seeker, ref mut anchor_seeker, ref mut upper_seeker) = self.meta;
        lower_seeker.method(offset).val.blerp(
            &upper_seeker.method(offset).val,
            anchor_seeker.method(offset)
        )
    }
}

impl<'a, T> Seekable<'a> for Automation<T>
where
    T: BoundLerp + Copy + Default + 'a

{
    type Seeker = AutomationSeeker<'a, T>;

    fn seeker(&'a self) -> Self::Seeker {
        Self::Seeker {
            meta: (self.lower.seeker(), self.anchors.seeker(),self.upper.seeker()),
            data: ()
        }
    }
}

impl BoundLerp for f32 {
    fn blerp(self, other: &Self, amount: f32) -> Self {
        self + (other - self) * amount
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
                        /*self.dimensions.y 
                            -*/ seeker.seek((x as f32 / res as f32) * self.dimensions.x)
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
