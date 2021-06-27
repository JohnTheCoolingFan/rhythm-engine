use crate::{foundation::automation::*, utils::misc_traits::*};
use ggez::graphics::Color;

pub struct ColorAnchor {
    color: Color,
    offset: f32,
}

impl ColorAnchor {
    pub fn new(c: Color, o: f32) -> Self {
        Self {
            color: c,
            offset: o,
        }
    }
}

pub struct DynColor {
    upper_colors: Vec<ColorAnchor>,
    automation: Automation,
    lower_colors: Vec<ColorAnchor>,
}

impl DynColor {
    pub fn new(len: f32) -> Self {
        Self {
            upper_colors: vec![ColorAnchor {
                color: Color::WHITE,
                offset: 0.,
            }],
            automation: Automation::new(0., 1., len, false),
            lower_colors: vec![ColorAnchor {
                color: Color::BLACK,
                offset: 0.,
            }],
        }
    }

    fn insert(vec: &mut Vec<ColorAnchor>, color_anch: ColorAnchor) {
        vec.insert(
            match vec.binary_search_by(|anch| anch.offset.partial_cmp(&color_anch.offset).unwrap())
            {
                Ok(index) => index,
                Err(index) => index,
            },
            color_anch,
        );
    }

    pub fn insert_upper(&mut self, color_anch: ColorAnchor) {
        Self::insert(&mut self.upper_colors, color_anch);
    }

    pub fn insert_lower(&mut self, color_anch: ColorAnchor) {
        Self::insert(&mut self.lower_colors, color_anch);
    }

    pub fn get_automation(&mut self) -> &mut Automation {
        &mut self.automation
    }
}

pub struct DynColorSeeker<'a> {
    upper_index: usize,
    lower_index: usize,
    automation_seeker: automation::AutomationSeeker<'a>,
    dyncolor: &'a DynColor,
}

impl<'a> DynColorSeeker<'a> {
    fn interp(&self, t: f32) -> Color {
        let c1 = self.dyncolor.lower_colors[if self.lower_index == self.dyncolor.lower_colors.len()
        {
            self.lower_index - 1
        } else {
            self.lower_index
        }]
        .color;

        let c2 = self.dyncolor.upper_colors[if self.upper_index == self.dyncolor.upper_colors.len()
        {
            self.upper_index - 1
        } else {
            self.upper_index
        }]
        .color;

        Color::new(
            (c2.r - c1.r) * t + c1.r,
            (c2.g - c1.g) * t + c1.g,
            (c2.b - c1.b) * t + c1.b,
            (c2.a - c1.a) * t + c1.a,
        )
    }
}

impl<'a> Seeker<Color> for DynColorSeeker<'a> {
    fn seek(&mut self, offset: f32) -> Color {
        while self.upper_index < self.dyncolor.upper_colors.len() {
            if offset <= self.dyncolor.upper_colors[self.upper_index].offset {
                break;
            }
            self.upper_index += 1;
        }
        self.upper_index -= if self.upper_index == 0 { 0 } else { 1 };

        while self.lower_index < self.dyncolor.lower_colors.len() {
            if offset <= self.dyncolor.lower_colors[self.lower_index].offset {
                break;
            }
            self.lower_index += 1;
        }
        self.lower_index -= if self.lower_index == 0 { 0 } else { 1 };

        let y = self.automation_seeker.seek(offset);
        self.interp(y)
    }

    fn jump(&mut self, val: f32) -> Color {
        self.upper_index = match self
            .dyncolor
            .upper_colors
            .binary_search_by(|anch| anch.offset.partial_cmp(&val).unwrap())
        {
            Ok(index) => index,
            Err(index) => {
                if index == 0 {
                    0
                } else {
                    index - 1
                }
            }
        };

        self.lower_index = match self
            .dyncolor
            .lower_colors
            .binary_search_by(|anch| anch.offset.partial_cmp(&val).unwrap())
        {
            Ok(index) => index,
            Err(index) => {
                if index == 0 {
                    0
                } else {
                    index - 1
                }
            }
        };

        println!("{}, {}", self.upper_index, self.lower_index);

        let y = self.automation_seeker.jump(val);
        self.interp(y)
    }
}

impl<'a> Seekable<'a> for DynColor {
    type Output = Color;
    type SeekerType = DynColorSeeker<'a>;
    fn seeker(&'a self) -> Self::SeekerType {
        Self::SeekerType {
            upper_index: 0,
            lower_index: 0,
            automation_seeker: self.automation.seeker(),
            dyncolor: &self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ggez::{
        event::{self, EventHandler, MouseButton},
        graphics::*,
        timer::time_since_start,
        Context, GameResult,
    };
    use glam::Vec2;

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

            test.color.insert_lower(ColorAnchor {
                color: Color::new(1., 0., 0., 1.),
                offset: x / 2.,
            });
            test.color.insert_upper(ColorAnchor {
                color: Color::new(0., 1., 0., 1.),
                offset: x * (2. / 3.),
            });
            test.color.insert_upper(ColorAnchor {
                color: Color::new(0., 1., 1., 1.),
                offset: x / 2.,
            });

            Ok(test)
        }
    }

    impl EventHandler for Test {
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
                    col.color,
                )?;

                draw(ctx, &rect, (Vec2::new(col.offset, 0.),))?;
            }

            for col in &self.color.upper_colors {
                let rect = Mesh::new_rectangle(
                    ctx,
                    DrawMode::fill(),
                    Rect::new(0., 0., self.dimensions.x, 20.),
                    col.color,
                )?;

                draw(
                    ctx,
                    &rect,
                    (Vec2::new(col.offset, self.dimensions.y - 20.),),
                )?;
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

            let mut auto_seeker = self.color.get_automation().seeker();
            let d = self.dimensions;
            let auto_points: Vec<Vec2> = (0..200)
                .map(|x| {
                    Vec2::new(
                        (x as f32 / 200.) * d.x,
                        auto_seeker.seek((x as f32 / 200.) * d.x) * d.y,
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
            let automation = self.color.get_automation();
            let index = automation.closest_to(ggez::input::mouse::position(ctx).into());
            match button {
                MouseButton::Left => {
                    automation.insert(Anchor::new(
                        Vec2::new(x, y / self.dimensions.y),
                        Weight::Curve(0.),
                    ));
                }
                MouseButton::Middle => {
                    automation.set_weight(
                        index,
                        match automation.get_weight(index) {
                            Weight::Curve(w) => {
                                if w != 0. {
                                    Weight::Curve(0.)
                                } else {
                                    Weight::ForwardBias
                                }
                            }
                            Weight::ForwardBias => Weight::ReverseBias,
                            Weight::ReverseBias => Weight::Curve(0.),
                        },
                    );
                }
                _ => {}
            }
        }

        fn mouse_wheel_event(&mut self, ctx: &mut Context, _x: f32, y: f32) {
            let automation = self.color.get_automation();
            let index = automation.closest_to(ggez::input::mouse::position(ctx).into());
            let weight = automation.get_weight(index);
            match weight {
                Weight::Curve(w) => automation
                    .set_weight(index, Weight::Curve(w + if 0. < y { 0.05 } else { -0.05 })),
                _ => {}
            };
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
