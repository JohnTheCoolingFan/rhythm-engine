mod core;
mod utils;

use ggez::{conf::WindowSetup, event, graphics::{Rect, drawable_size, set_screen_coordinates}};
use ggez::graphics::{self, Color};
use ggez::{Context, GameResult};
use glam::*;
use crate::core::curves;

struct MainState {
}

impl MainState {
    fn new() -> GameResult<MainState> {
        Ok(MainState {})
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());
        let (width, height) = drawable_size(ctx);
        set_screen_coordinates(ctx, Rect{x: 0., y: 0., w: width, h: height })?;
        let pos = ggez::input::mouse::position(ctx);

        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            Vec2::new(pos.x, pos.y),
            5.0,
            2.0,
            Color::new(1.0, 1.0, 1.0, 1.0),
        )?;
        graphics::draw(ctx, &circle, (Vec2::new(0., 0.),))?;

        graphics::present(ctx)?;
        Ok(())
    }
}

pub fn main() -> GameResult {
    /*let cb = ggez::ContextBuilder::new("Rythm Engine", "iiYese");
    let (mut ctx, event_loop) = cb.build()?;
    let state = MainState::new()?;
    graphics::set_resizable(&mut ctx, true)?;
    event::run(ctx, event_loop, state)*/
    curves::tests::run()
}
