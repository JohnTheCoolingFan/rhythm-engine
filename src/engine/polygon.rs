use crate::engine::renderer::VertexCtor;

use super::renderer::{Color, Vertex, Tesselate};
use glam::Vec2;
use lyon::{
    math::point,
    path::polygon::Polygon as LyonPolygon,
    tessellation::{ FillTessellator, VertexBuffers }, lyon_tessellation::{ BuffersBuilder, FillOptions },
};



pub struct Polygon {
    points: Vec<Vec2>,
    primary_color: Color,
    secondary_color: Option<(Color, usize, usize)>,
    bloom: f32,
    z_offset: f32,
}

impl Polygon {
    pub fn new(scale_factor: f32) -> Polygon {
        Polygon {
            points: vec![
                Vec2::new(0.0, 1.0) * scale_factor,
                Vec2::new(1.0, 0.0) * scale_factor,
                Vec2::new(-1.0, 0.0) * scale_factor,
            ],
            primary_color: Color::new(0.5, 0.6, 0.7, 1.0),
            secondary_color: None,
            bloom: 0.0,
            z_offset: 0.0,
        }
    }

    pub fn contains(&self, point: Vec2) -> bool {
        let mut inside = false;

        for i in 0..self.points.len() {
            let p1 = self.points[i];
            let p2 = self.points[(i + 1) % self.points.len()];
            
            if  (point.y < p1.y) != 
                (point.y < p2.y) && 
                point.x < (p2.x - p1.x) * (point.y - p1.y) / (p2.y - p1.y) + p1.x 
            {
                inside = !inside;
            }
        }

        inside
    }
}

impl Tesselate for Polygon {
    fn tesselate(&self) -> VertexBuffers<Vertex, u32> {
        debug_assert!(self.points.len() >= 3);
        let attrs = [
            self.z_offset,
            self.primary_color.r,
            self.primary_color.g,
            self.primary_color.b,
            self.primary_color.a
        ];
        
        let mut path_builder = lyon::path::Path::builder_with_attributes(4);
        path_builder.begin(point(self.points[0].x, self.points[0].y), &attrs);
        for p in &self.points[1..] {
            path_builder.line_to(point(p.x, p.y), &attrs);
        }
        path_builder.close();

        let mut buffers: VertexBuffers<Vertex, u32> = VertexBuffers::new();
        let mut vertex_builder = BuffersBuilder::new(&mut buffers, VertexCtor);

        let mut tessellator = FillTessellator::new();
        let _ = tessellator.tessellate_polygon(
            LyonPolygon {
                points: self.points.iter()
                    .map(|p| point(p.x, p.y))
                    .collect::<Vec<_>>()
                    .as_slice(),
                closed: true,
            },
            &FillOptions::default(),
            &mut vertex_builder,
        );

        if let Some((secondary_color, start, size)) = self.secondary_color {
            debug_assert!(start + size <= self.points.len());
            for vertex in &mut buffers.vertices {
                vertex.color = secondary_color.into();
            }
        }

        buffers
    }
}
