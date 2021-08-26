use std::ops::{Deref, DerefMut, Add, Sub, Mul};
use duplicate::*;
use glam::{Vec2, Mat3};
use super::{automation::*, anchor::*};
use crate::utils::seeker::*;

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
            s,      0.,     pivot.x - s * pivot.x,
            0.,     s,      pivot.y - s * pivot.y,
            0.,     0.,     1.
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
    use ggez::{
        event::{self, EventHandler, MouseButton, KeyCode},
        graphics::*,
        Context,
        GameError,
        GameResult
    };
    struct Test {
        rotation: TransformPoint<Rotation>,
        scale: TransformPoint<Scale>,
        dimensions: Vec2
    }

    impl Test {
        fn new() -> GameResult<Self> {
            let dimensions = Vec2::new(2000., 1000.);
            Ok(Self{
                rotation: TransformPoint::<Rotation>{
                    automation: Automation::<Rotation>::new(Rotation(0.), Rotation(180.), dimensions.x),
                    point: Some(Vec2::new(dimensions.x / 3., dimensions.y / 2.))
                },
                scale: TransformPoint::<Scale>{
                    automation: Automation::<Scale>::new(Scale(1.), Scale(3.), dimensions.x),
                    point: Some(Vec2::new(dimensions.x * (2. / 3.), dimensions.y / 2.))
                },
                dimensions
            })
        }
    }

    impl EventHandler<GameError> for Test {
        fn update(&mut self, _ctx: &mut Context) -> GameResult {
            Ok(())
        }

        fn draw(&mut self, ctx: &mut Context) -> GameResult {
            let rect = Mesh::new_rectangle(
                ctx,
                DrawMode::fill(),
                Rect::new(-5., -5., 10., 10.),
                Color::new(1., 0., 0., 0.5),
            )?;

            draw(ctx, &rect, (self.rotation.point.unwrap(),))?;
            draw(ctx, &rect, (self.scale.point.unwrap(),))?;


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
                                * transform.seek((x as f32 / res as f32) * self.dimensions.x)
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


            present(ctx)?;
            Ok(())
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
