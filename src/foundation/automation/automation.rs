use crate::utils::{seeker::*, FromEnd};
use glam::Vec2;

#[derive(Debug, Copy, Clone)]
pub enum Weight {
    ForwardBias,
    Curve(f32),
    ReverseBias,
}

pub struct Anchor {
    point: Vec2,
    weight: Weight,
}

impl Anchor {
    pub fn new(p: Vec2, w: Weight) -> Self {
        Self {
            point: p,
            weight: w,
        }
    }
}

pub struct Automation {
    upper_bound: f32,
    lower_bound: f32,
    dynamic_bound: bool,

    anchors: Vec<Anchor>,
}

impl Automation {
    pub fn new(lb: f32, ub: f32, len: f32, dynamic: bool) -> Self {
        Automation {
            upper_bound: ub,
            lower_bound: lb,
            dynamic_bound: dynamic,
            anchors: vec![
                Anchor::new(Vec2::new(0., 0.0), Weight::Curve(0.)),
                Anchor::new(Vec2::new(len, 0.0), Weight::Curve(0.)),
            ],
        }
    }

    pub fn get_upper_bound(&self) -> f32 {
        self.upper_bound
    }

    pub fn get_lower_bound(&self) -> f32 {
        self.lower_bound
    }

    pub fn set_upper_bound(&mut self, v: f32) {
        if self.dynamic_bound {
            self.upper_bound = v;
        }
    }

    pub fn set_lower_bound(&mut self, v: f32) {
        if self.dynamic_bound {
            self.lower_bound = v;
        }
    }

    pub fn len(&self) -> usize {
        self.anchors.len()
    }

    pub fn insert(&mut self, item: Anchor) {
        self.anchors.insert(
            match self
                .anchors
                .binary_search_by(|elem| elem.point.x.partial_cmp(&item.point.x).unwrap())
            {
                Ok(index) => index,
                Err(index) => index,
            },
            item,
        );
    }

    pub fn remove(&mut self, index: usize) -> Anchor {
        self.anchors.remove(index)
    }

    pub fn closest_to(&mut self, point: Vec2) -> usize {
        let (index, _) = self
            .anchors
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                (a.point - point)
                    .length()
                    .partial_cmp(&(b.point - point).length())
                    .unwrap()
            })
            .unwrap();

        index
    }

    pub fn get_pos(&self, index: usize) -> Vec2 {
        self.anchors[index].point
    }

    pub fn set_pos(&mut self, index: usize, mut point: Vec2) {
        let minx = if index == 0 {
            0.
        } else {
            self.anchors[index - 1].point.x
        };
        let maxx = if self.anchors.len() - index == 1 {
            self.anchors[FromEnd(0)].point.x
        } else {
            self.anchors[index + 1].point.x
        };

        point.x = point.x.clamp(minx, maxx);
        point.y = point.y.clamp(0., 1.);
        self.anchors[index].point = point;
    }

    pub fn get_weight(&self, index: usize) -> Weight {
        self.anchors[index].weight
    }

    pub fn set_weight(&mut self, index: usize, weight: Weight) {
        self.anchors[index].weight = match weight {
            Weight::Curve(w) => Weight::Curve(if 0. <= w {
                w.clamp(0., 30.)
            } else {
                w.clamp(-30., 0.)
            }),
            _ => weight,
        };
    }

    pub fn get_span(&self) -> f32 {
        self.anchors[FromEnd(0)].point.x
    }
}

pub struct AutomationSeeker<'a> {
    index: usize,
    automation: &'a Automation,
}

impl<'a> AutomationSeeker<'a> {
    //lower bound upper bound val
    fn from_y(&self, y: f32) -> f32 {
        debug_assert!(0. <= y && y <= 1.);
        self.automation.lower_bound
            + (self.automation.upper_bound - self.automation.lower_bound) * y
    }

    pub fn interp(&self, offset: f32) -> f32 {
        self.from_y(if 0 == self.index {
            self.automation.anchors[0].point.y
        } else if self.index == self.automation.anchors.len() {
            let anch = &self.automation.anchors[FromEnd(0)];
            match anch.weight {
                Weight::ReverseBias => self.automation.anchors[FromEnd(1)].point.y,
                _ => anch.point.y,
            }
        } else {
            let start = &self.automation.anchors[self.index - 1];
            let end = &self.automation.anchors[self.index];

            let t = (offset - start.point.x) / (end.point.x - start.point.x);

            match end.weight {
                Weight::ReverseBias => start.point.y,
                Weight::Curve(w) => {
                    start.point.y
                        + (end.point.y - start.point.y)
                            * t.powf(if w < 0. { 1. / (w.abs() + 1.) } else { w + 1. })
                }
                Weight::ForwardBias => end.point.y,
            }
        })
    }
}

impl<'a> Seeker<f32> for AutomationSeeker<'a> {
    fn seek(&mut self, offset: f32) -> f32 {
        while self.index < self.automation.anchors.len() {
            if offset < self.automation.anchors[self.index].point.x {
                break;
            }
            self.index += 1;
        }
        self.interp(offset)
    }

    fn jump(&mut self, offset: f32) -> f32 {
        match self
            .automation
            .anchors
            .binary_search_by(|t| t.point.x.partial_cmp(&offset).unwrap())
        {
            Ok(index) => {
                self.index = index;
                self.from_y(self.automation.anchors[index].point.y)
            }
            Err(index) => {
                self.index = index;
                self.interp(offset)
            }
        }
    }
}

impl<'a> Seekable<'a, f32> for Automation {
    type Seeker = AutomationSeeker<'a>;
    fn seeker(&'a self) -> Self::Seeker {
        Self::Seeker {
            index: 0,
            automation: &self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ggez::{
        event::{self, EventHandler, MouseButton},
        graphics::*,
    };
    use ggez::{Context, GameResult};

    struct AutomationTest {
        automation: Automation,
        dimensions: Vec2,
    }

    impl AutomationTest {
        fn new() -> GameResult<Self> {
            Ok(Self {
                automation: Automation::new(0., 1., 2800., false),
                dimensions: Vec2::new(2800., 1100.),
            })
        }
    }

    impl EventHandler for AutomationTest {
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
            let res = 200;
            let points: Vec<Vec2> = (0..res)
                .map(|x| {
                    Vec2::new(
                        (x as f32 / res as f32) * self.dimensions.x,
                        seeker.seek((x as f32 / res as f32) * self.dimensions.x)
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
                    self.automation.insert(Anchor {
                        point: Vec2::new(x, y / self.dimensions.y),
                        weight: Weight::Curve(0.),
                    });
                }
                MouseButton::Middle => {
                    self.automation.set_weight(
                        index,
                        match self.automation.get_weight(index) {
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
            let index = self
                .automation
                .closest_to(ggez::input::mouse::position(ctx).into());
            let weight = self.automation.get_weight(index);
            match weight {
                Weight::Curve(w) => self
                    .automation
                    .set_weight(index, Weight::Curve(w + if 0. < y { 0.05 } else { -0.05 })),
                _ => {}
            };
        }
    }

    #[test]
    pub fn automation() -> GameResult {
        let state = AutomationTest::new()?;
        let cb = ggez::ContextBuilder::new("Automation test", "iiYese").window_mode(
            ggez::conf::WindowMode::default().dimensions(state.dimensions.x, state.dimensions.y),
        );
        let (ctx, event_loop) = cb.build()?;
        event::run(ctx, event_loop, state)
    }
}
