use super::renderer::{Color, Vertex};
use glam::Vec2;



pub struct Polygon {
    points: Vec<Vec2>,
    primary_color: Color,
    secondary_color: Option<(Color, u32, u32)>,
    layer: u32,
}

impl Polygon {
    pub fn new(scale_factor: f32) -> Polygon {
        Polygon {
            points: vec![
                Vec2::new(0.0, 1.0) * scale_factor,
                Vec2::new(1.0, 0.0) * scale_factor,
                Vec2::new(0.0, -1.0) * scale_factor,
            ],
            primary_color: Color::new(0.0, 0.0, 0.0, 1.0),
            secondary_color: None,
            layer: 0,
        }
    }

    pub fn triangulate(&self) -> Vec<Vertex> {
        unimplemented!()
    }
}
