use crate::utils::seeker::*;
use glam::Vec2;

pub struct Anchor {
    point: Vec2,
    power: f32,
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
                    power: 1.,
                },
                Anchor {
                    point: Vec2::new(len, 0.0),
                    power: 1.,
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

    pub fn push(&mut self, anchor: Anchor) {
        self.anchors.push(anchor);
    }

    pub fn pop(&mut self) -> Anchor {
        self.anchors.pop().unwrap()
    }

    pub fn remove(&mut self, index: usize) -> Anchor {
        self.anchors.remove(index)
    }
}

pub struct AutomationSeeker<'a> {
    index: usize,
    automantion: &'a Automation,
}

impl<'a> AutomationSeeker<'a> {
    //lower bound upper bound val
    fn y_to_lbub_val(&self, y: f32) -> f32 {
        debug_assert!(0. <= y && y <= 1.);
        self.automantion.lower_bound
            + (self.automantion.upper_bound - self.automantion.lower_bound) * y
    }

    fn interp(&self, offset: f32) -> f32 {
        debug_assert!(0 < self.index && self.index < self.automantion.anchors.len());
        debug_assert!(
            self.automantion.anchors[self.index - 1].point.x <= offset
                && offset <= self.automantion.anchors[self.index].point.x
        );

        let start = &self.automantion.anchors[self.index - 1];
        let end = &self.automantion.anchors[self.index];

        let t = (offset - self.automantion.anchors[self.index - 1].point.x)
            / (self.automantion.anchors[self.index].point.x
                - self.automantion.anchors[self.index - 1].point.x);

        self.y_to_lbub_val(
            start.point.y 
            + (end.point.y - start.point.y) 
            * t.powf(if 0. < end.power { end.power } else { 1. / end.power.abs() })
        )
    }
}

impl<'a> Seeker<f32> for AutomationSeeker<'a> {
    fn seek(&mut self, offset: f32) -> f32 {
        while self.index < self.automantion.anchors.len() {
            if offset == self.automantion.anchors[self.index].point.x {
                return self.y_to_lbub_val(self.automantion.anchors[self.index].point.y);
            } else if offset < self.automantion.anchors[self.index].point.x {
                break;
            }
            self.index += 1;
        }
        if 0 == self.index {
            self.y_to_lbub_val(self.automantion.anchors[0].point.y)
        } else {
            self.interp(offset)
        }
    }

    fn jump(&mut self, offset: f32) -> f32 {
        match self
            .automantion
            .anchors
            .binary_search_by(|t| t.point.x.partial_cmp(&offset).unwrap())
        {
            Ok(index) => {
                self.index = index;
                self.y_to_lbub_val(self.automantion.anchors[index].point.y)
            }
            Err(index) => {
                self.index = index;
                if 0 == index || index == self.automantion.anchors.len() {
                    self.y_to_lbub_val(self.automantion.anchors[index].point.y)
                } else {
                    self.interp(offset)
                }
            }
        }
    }
}

impl <'a> Seekable<'a, f32> for Automation {
    type Seeker = AutomationSeeker<'a>;
    fn seeker(&'a self) -> Self::Seeker {
        Self::Seeker {
            index: 0,
            automantion: &self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ggez::graphics::*;
    use ggez::{
        event::{self, EventHandler, KeyCode, KeyMods, MouseButton},
        graphics::MeshBuilder,
    };
    use ggez::{Context, GameResult};
    use crate::utils::from_end::FromEnd;

    struct AutomationTest {
        auto: Automation,
        dimensions: Vec2
    }
   
    impl AutomationTest {
        fn new() -> GameResult<Self> {
            Ok(Self {
                auto: Automation::new(0., 1., 30., false),
                dimensions: Vec2::new(2800., 1100.)
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
                    Color::new(1., 1., 1., 1.)
                )?.build(ctx)?;
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
                    self.auto.push(Anchor{ point: Vec2::new(x,  y / self.dimensions.y), power: 1.});
                    println!("{:?}", self.auto.anchors[FromEnd(0)].point);
                }
                _ => {}
            }
        }

        fn key_down_event(&mut self, _ctx: &mut Context, key: KeyCode, _keymods: KeyMods, _repeat: bool) {
            self.auto.anchors[FromEnd(0)].power += match key {
                KeyCode::Up => 0.05,
                KeyCode::Down => -0.05,
                _ => 0.
            }
        }
    }

    #[test]
    pub fn automation() -> GameResult {
        let state = AutomationTest::new()?;
        let cb = ggez::ContextBuilder::new("Automation test", "iiYese")
            .window_mode(
                ggez::conf::WindowMode::default().dimensions(state.dimensions.x, state.dimensions.y)
            );
        let (ctx, event_loop) = cb.build()?;
        event::run(ctx, event_loop, state)
    }
}
