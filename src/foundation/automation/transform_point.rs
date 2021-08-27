use std::ops::{Deref, DerefMut, Add, Sub, Mul};
use duplicate::*;
use glam::{Vec2, Mat3};
use super::{automation::*, anchor::*};
use crate::utils::*;

duplicate_inline! {
    [T; [Rotation]; [Scale]] //more stuff like shear, pinch, explode later
    
    #[derive(Clone, Copy)]
    pub struct T(pub f32);
    
    impl Deref for T {
        type Target = f32;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    
    impl DerefMut for T {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    impl From<f32> for T {
        fn from(v: f32) -> Self {
            Self(v)
        }
    }

    impl BoundLerp for T {
        fn blerp(self, other: Self, amount: f32) -> Self {
            Self(self.0 + (other.0 - self.0) * amount)
        }
    }
}
//
//
//
//
//
pub trait TransformDictator: Copy + Deref<Target = f32> + From<f32> {}
impl<T> TransformDictator for T 
where
    T: Copy + Deref<Target = f32> + From<f32>
{}

pub struct CrudeTransform<T>
where
    Mat3: From<Self>,
    T: TransformDictator
{
    pub factor: T,
    pub pivot: Vec2
}

pub enum Transform<T>
where
    Mat3: From<CrudeTransform<T>>,
    T: TransformDictator
{
    Pre(T, Option<Vec2>),
    Post(Mat3)
}

impl<T> Transform<T> 
where
    Mat3: From<CrudeTransform<T>>,
    T: TransformDictator
{
    pub fn process(&mut self, auxiliary: &Vec2) -> &Mat3 {
        match self {
            Self::Pre(factor, point) =>
                *self = Self::Post(CrudeTransform{factor:*factor, pivot:
                    if let Some(p) = point { *p }
                    else { *auxiliary }
                }.into()),
            Self::Post(ref transform) => return transform
        }
        self.process(auxiliary)
    }
}
//
//
//
//
//
impl From<CrudeTransform<Rotation>> for Mat3 {
    #[rustfmt::skip]
    fn from(CrudeTransform{ factor, pivot }: CrudeTransform<Rotation>) -> Self {
        let r = factor.to_radians();
        let (x, y) = (pivot.x, pivot.y);
        let (r10, r11) = r.sin_cos();
        let (r00, r01) = (r11, -r10);

        Mat3::from_cols_array(&[
            r00,    r01,    x - r00 * x - r01 * y,
            r10,    r11,    y - r10 * x - r11 * y,
            0.,      0.,    1.,
        ]).transpose()
    }
}

impl From<CrudeTransform<Scale>> for Mat3 {
    #[rustfmt::skip]
    fn from(CrudeTransform{ factor, pivot }: CrudeTransform<Scale>) -> Self {
        let s = factor.0;
        Mat3::from_cols_array(&[
            s,                      0.,                     0.,
            0.,                     s,                      0.,
            pivot.x - s * pivot.x,  pivot.y - s * pivot.y,  1.
        ])
    }
}
//
//
//
//
//
pub struct TransformPoint<T> 
where
    Mat3: From<CrudeTransform<T>>,
    T: TransformDictator
{
    pub automation: Automation<T>,
    pub point: Option<Vec2>
}

pub type TransformPointSeeker<'a, T> = Seeker<&'a Option<Vec2>, AutomationSeeker<'a, T>>;

impl<'a, T> SeekerTypes for TransformPointSeeker<'a, T>
where
    Mat3: From<CrudeTransform<T>>,
    T: TransformDictator
{
    type Source = Anchor;
    type Output = Transform<T>;
}

impl<'a, T> Seek for TransformPointSeeker<'a, T>
where
    Mat3: From<CrudeTransform<T>>,
    T: TransformDictator + BoundLerp

{
    #[duplicate(method; [seek]; [jump])]
    fn method(&mut self, offset: f32) -> Transform<T> {
        Transform::<T>::Pre(
            self.meta.method(offset).into(),
            *self.data
        )
    }
}

impl <'a, T> Seekable<'a> for TransformPoint<T>
where
    Mat3: From<CrudeTransform<T>>,
    T: TransformDictator + BoundLerp
{
    type Seeker = TransformPointSeeker<'a, T>;
    fn seeker(&'a self) -> Self::Seeker {
        Self::Seeker{
            data: &self.point,
            meta: self.automation.seeker()
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
    use std::cmp::Ordering;
    use ggez::{
        event::{self, EventHandler, MouseButton, KeyCode, KeyMods},
        graphics::*,
        timer::time_since_start,
        Context,
        GameError,
        GameResult
    };
    use crate::foundation::automation::automation::tests::key_handle;
    use glam::Vec3;

    struct Test {
        rotation: TransformPoint<Rotation>,
        scale: TransformPoint<Scale>,
        dimensions: Vec2,
        point_cache: Option<Vec2>
    }

    impl Test {
        fn new() -> GameResult<Self> {
            let dimensions = Vec2::new(2000., 1000.);
            Ok(Self{
                rotation: TransformPoint::<Rotation>{
                    automation: Automation::<Rotation>::new(Rotation(0.), Rotation(360.), dimensions.x),
                    point: Some(Vec2::new(dimensions.x / 3., dimensions.y / 2.))
                },
                scale: TransformPoint::<Scale>{
                    automation: Automation::<Scale>::new(Scale(1.), Scale(3.), dimensions.x),
                    point: Some(Vec2::new(dimensions.x * (2. / 3.), dimensions.y / 2.))
                },
                dimensions,
                point_cache: None
            })
        }
    }

    impl EventHandler<GameError> for Test {
        fn update(&mut self, _ctx: &mut Context) -> GameResult {
            Ok(())
        }

        fn draw(&mut self, ctx: &mut Context) -> GameResult {
            clear(ctx, Color::new(0., 0., 0., 1.));
            let rect = Mesh::new_rectangle(
                ctx,
                DrawMode::fill(),
                Rect::new(-5., -5., 10., 10.),
                Color::new(1., 0., 0., 0.5),
            )?;

            for point in IntoIterator::into_iter([&self.rotation.point, &self.scale.point]).flatten() {
                draw(ctx, &rect, (*point,))?;
            }

            let res = 500;
            duplicate_inline! {
                [
                    transform       lines               points              n;
                    [scale]         [scale_lines]       [scale_points]      [0.];
                    [rotation]      [rotation_lines]    [rotation_points]   [1.];
                ]
                let mut transform = self.transform.automation.anchors.seeker();
                let points: Vec<Vec2> = (0..res)
                    .map(|x| {
                        Vec2::new(
                            (x as f32 / res as f32) * self.dimensions.x,
                            self.dimensions.y - (self.dimensions.y * n * 0.15) - (
                                0.15 
                                * self.dimensions.y
                                * (1. - transform.jump(
                                    (x as f32 / res as f32) * self.dimensions.x
                                ))
                            )
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
            }

            let t = self.dimensions.x * (
                (time_since_start(ctx).as_millis() as f32 % 5000.)
                / 5000.
            );

            let t_line = MeshBuilder::new()
                .polyline(
                    DrawMode::Stroke(StrokeOptions::DEFAULT),
                    &[
                        Vec2::new(0., self.dimensions.y * 0.7),
                        Vec2::new(0., self.dimensions.y),
                    ],
                    Color::WHITE,
                )?
                .build(ctx)?;

            draw(ctx, &t_line, (Vec2::new(t, 0.),))?;

            let center = Vec2::new(0.5 * self.dimensions.x, 0.5 * self.dimensions.y);

            let rect =[
                center + Vec2::new(-20., 20.), center + Vec2::new(20., 20.),
                center + Vec2::new(20., -20.), center + Vec2::new(-20., -20.) 
            ];

            let mut scale = self.scale.seeker().jump(t);
            let s = scale.process(&center);
            let mut rotate = self.rotation.seeker().jump(t);
            let r = rotate.process(&center);

            let scaled: Vec<Vec2> = rect.iter().map(
                |p| -> Vec2 {
                    let v3 = *s * p.extend(1.);
                    (v3.x, v3.y).into()
                }
            ).collect();

            let rotated: Vec<Vec2> = rect.iter().map(
                |p| -> Vec2 {
                    let v3 = *r * p.extend(1.);
                    (v3.x, v3.y).into()
                }
            ).collect();


            let r0 = MeshBuilder::new()
                .polygon(
                    DrawMode::Fill(FillOptions::DEFAULT),
                    &rect,
                    Color::BLUE
                )?
                .build(ctx)?;


            let r1 = MeshBuilder::new()
                .polygon(
                    DrawMode::Fill(FillOptions::DEFAULT),
                    scaled.as_slice(),
                    Color::CYAN
                )?
                .build(ctx)?;

            let r2 = MeshBuilder::new()
                .polygon(
                    DrawMode::Fill(FillOptions::DEFAULT),
                    rotated.as_slice(),
                    Color::GREEN
                )?
                .build(ctx)?;


            draw(ctx, &r0, (Vec2::new(0., 0.),))?;
            draw(ctx, &r1, (Vec2::new(0., 0.),))?;
            draw(ctx, &r2, (Vec2::new(0., 0.),))?;

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
            let pos = Vec2::new(x, y);
            if y < self.dimensions.y - self.dimensions.y * 2. * 0.15 {
                match button {
                    MouseButton::Left => self.point_cache = Some(pos),
                    MouseButton::Right => {
                        let closest: &mut Option<Vec2> = 
                            //this is stoopid https://github.com/rust-lang/rust/issues/25725
                            IntoIterator::into_iter([&mut self.rotation.point, &mut self.scale.point])
                                .min_by(|a, b| { match (a, b) {
                                    (None, Some(_)) => Ordering::Greater,
                                    (Some(p0), Some(p1)) =>
                                        (*p0 - pos)
                                            .length()
                                            .partial_cmp(&(*p1 - pos).length())
                                            .unwrap(),
                                    _ => Ordering::Less
                                }})
                                .unwrap();

                        if let Some(ref mut p) = closest {
                            if (*p - pos).length() < 10. {
                                *closest = None;
                            }
                        }
                    }
                    _ => {}
                }
            }
            else {
                //Generic closures not possible yet :pensive:
                duplicate_inline! {
                    [
                        handle              T;
                        [rotation_handle]   [Rotation];
                        [scale_handle]      [Scale];
                    ]
                    let handle = |auto: &mut Automation<T>, adjusted_y: f32| {
                        let index = auto.closest_to(ggez::input::mouse::position(ctx).into());

                        match button {
                            MouseButton::Left => { auto.insert(Anchor::new(Vec2::new(x, adjusted_y))); },
                            MouseButton::Middle => { auto[index].weight.cycle(); },
                            MouseButton::Right => { auto.remove(index); },
                            _ => {}
                        }
                    };
                }
                if pos.y < self.dimensions.y - self.dimensions.y * 1. * 0.15 {
                    let adj_y = 
                        (y - (self.dimensions.y - self.dimensions.y * 2. * 0.15)) 
                        / (self.dimensions.y * 0.15);
                    rotation_handle(&mut self.rotation.automation, adj_y);
                } else {
                    let adj_y = 
                        (y - (self.dimensions.y - self.dimensions.y * 1. * 0.15)) 
                        / (self.dimensions.y * 0.15);
                    scale_handle(&mut self.scale.automation, adj_y);
                };
            }
        }

        fn mouse_wheel_event(&mut self, ctx: &mut Context, _x: f32, y: f32) {
            let pos: Vec2 = ggez::input::mouse::position(ctx).into();
            if pos.y < self.dimensions.y - self.dimensions.y * 2. * 0.15 { return }
            duplicate_inline! {
                [
                    handle              T;
                    [rotation_handle]   [Rotation];
                    [scale_handle]      [Scale];
                ]
                let handle = |auto: &mut Automation<T>| {
                    let index = auto.closest_to(pos);
                    let _ = auto[index].weight.shift_curvature(
                        if 0. < y { 0.05 } else { -0.05 }
                    );
                };
            }
            if pos.y < self.dimensions.y - self.dimensions.y * 1. * 0.15 {
                rotation_handle(&mut self.rotation.automation);
            } else {
                scale_handle(&mut self.scale.automation);
            }
        }

        fn key_down_event(&mut self, ctx: &mut Context, key: KeyCode, _mods: KeyMods, _: bool) {
            let pos: Vec2 = ggez::input::mouse::position(ctx).into();
            if pos.y < self.dimensions.y - self.dimensions.y * 2. * 0.15 { return }
            duplicate_inline! {
                [
                    handle              T;
                    [rotation_handle]   [Rotation];
                    [scale_handle]      [Scale];
                ]
                let handle = |auto: &mut Automation<T>| {
                    let index = auto.closest_to(ggez::input::mouse::position(ctx).into());
                    key_handle(
                        &mut auto[index],
                        key
                    );
                };
            }
            if pos.y < self.dimensions.y - self.dimensions.y * 1. * 0.15 {
                rotation_handle(&mut self.rotation.automation);
            } else {
                scale_handle(&mut self.scale.automation);
            }

        }

    }

    #[test]
    pub fn transform_point() -> GameResult {
        let state = Test::new()?;
        let cb = ggez::ContextBuilder::new("Transform Point test", "iiYese").window_mode(
            ggez::conf::WindowMode::default().dimensions(state.dimensions.x, state.dimensions.y),
        );
        let (ctx, event_loop) = cb.build()?;
        event::run(ctx, event_loop, state)
    }
}
