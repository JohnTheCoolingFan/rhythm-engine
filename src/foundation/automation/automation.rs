use crate::utils::{seeker::*, FromEnd};
use glam::Vec2;

pub struct Anchor {
    point: Vec2,
    weight: f32,
}

impl Anchor {
    pub fn new(p: Vec2, w: f32) -> Self {
        Self {
            point: p,
            weight: w
        }
    }
}

impl Anchor {
    pub fn add_to_weight(&mut self, value: f32) {
        let new_weight = self.weight + value;
        self.weight = || -> f32 {
            for i in -1..=1 {
                let f = i as f32;
                if (self.weight < f && f <= new_weight) || (new_weight <= f && f < self.weight) {
                    return f;
                }
            }
            if (self.weight == -1. || self.weight == 0.) && 0. < value {
                self.weight + 1.
            } else if (self.weight == 0. || self.weight == 1.) && value < 0. {
                self.weight - 1.
            } else {
                if new_weight.abs() < 20. {
                    new_weight
                } else {
                    self.weight
                }
            }
        }();
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
                Anchor {
                    point: Vec2::new(0., 0.0),
                    weight: 1.,
                },
                Anchor {
                    point: Vec2::new(len, 0.0),
                    weight: 1.,
                },
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

    pub fn closest_to(&mut self, point: Vec2) -> &mut Anchor {
        let (index, _) = self.anchors
            .iter()
            .enumerate()
            .min_by(
                |(_, a), (_, b)| 
                    (a.point - point)
                    .length()
                    .partial_cmp(&(b.point - point).length()).unwrap()
            ).unwrap();

        &mut self.anchors[index]
    }
}

pub struct AutomationSeeker<'a> {
    index: usize,
    automantion: &'a Automation,
}

impl<'a> AutomationSeeker<'a> {
    //lower bound upper bound val
    fn from_y(&self, y: f32) -> f32 {
        debug_assert!(0. <= y && y <= 1.);
        self.automantion.lower_bound
            + (self.automantion.upper_bound - self.automantion.lower_bound) * y
    }

    pub fn interp(&self, offset: f32) -> f32 {
        self.from_y(if self.index == self.automantion.anchors.len() {
            self.automantion.anchors[FromEnd(0)].point.y
        } else if self.index == 0 {
            self.automantion.anchors[0].point.y
        } else {
            let start = &self.automantion.anchors[self.index - 1];
            let end = &self.automantion.anchors[self.index];

            if end.weight == 0. {
                start.point.y
            } else {
                let t = (offset - self.automantion.anchors[self.index - 1].point.x)
                    / (self.automantion.anchors[self.index].point.x
                        - self.automantion.anchors[self.index - 1].point.x);

                start.point.y
                    + (end.point.y - start.point.y)
                        * t.powf(if 0. < end.weight {
                            end.weight
                        } else {
                            1. / end.weight.abs()
                        })
            }
        })
    }
}

impl<'a> Seeker<f32> for AutomationSeeker<'a> {
    fn seek(&mut self, offset: f32) -> f32 {
        while self.index < self.automantion.anchors.len() {
            if offset < self.automantion.anchors[self.index].point.x {
                break;
            }
            self.index += 1;
        }
        self.interp(offset)
    }

    fn jump(&mut self, offset: f32) -> f32 {
        match self
            .automantion
            .anchors
            .binary_search_by(|t| t.point.x.partial_cmp(&offset).unwrap())
        {
            Ok(index) => {
                self.index = index;
                self.from_y(self.automantion.anchors[index].point.y)
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
            automantion: &self,
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
        auto: Automation,
        dimensions: Vec2,
    }

    impl AutomationTest {
        fn new() -> GameResult<Self> {
            Ok(Self {
                auto: Automation::new(0., 1., 2800., false),
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

            let mut points = Vec::<Vec2>::new();
            let mut t = 0.;

            let mut seeker = self.auto.seeker();
            while t < self.auto.anchors[FromEnd(0)].point.x {
                points.push(Vec2::new(t, seeker.seek(t) * self.dimensions.y));
                t += 1.;
            }

            let lines = MeshBuilder::new()
                .polyline(
                    DrawMode::Stroke(StrokeOptions::DEFAULT),
                    points.as_slice(),
                    Color::new(1., 1., 1., 1.),
                )
                ?.build(ctx)?;
            draw(ctx, &lines, (Vec2::new(0.0, 0.0),))?;

            present(ctx)?;
            Ok(())
        }

        fn mouse_button_down_event(
            &mut self,
            _ctx: &mut Context,
            button: MouseButton,
            x: f32,
            y: f32,
        ) {
            match button {
                MouseButton::Left => {
                    println!("left click");
                    self.auto.insert(Anchor {
                        point: Vec2::new(x, y / self.dimensions.y),
                        weight: 1.,
                    });
                }
                _ => {}
            }
        }

        fn mouse_wheel_event(&mut self, ctx: &mut Context, _x: f32, y: f32) {
            self.auto.closest_to(ggez::input::mouse::position(ctx).into()).add_to_weight(
                if y < 0. {
                    0.01
                }
                else if 0. < y {
                    -0.01
                }
                else {
                    0.
                }
            );
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
