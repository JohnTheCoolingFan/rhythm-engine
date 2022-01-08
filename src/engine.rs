pub mod renderer;
pub mod polygon;

use lyon::tessellation::{FillVertexConstructor, FillVertex};
#[derive(Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl From<Color> for [f32; 4] {
    fn from(color: Color) -> Self {
        [color.r, color.g, color.b, color.a]
    }
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

pub struct VertexCtor;
impl FillVertexConstructor<Vertex> for VertexCtor {
    fn new_vertex(&mut self, mut vertex: FillVertex) -> Vertex {
        let position = vertex.position();
        let attrs = vertex.interpolated_attributes();
        Vertex {
            position: [position.x, position.y, attrs[0]],
            color: [ attrs[1], attrs[2], attrs[3], attrs[4] ],
        }
    }
}

impl Vertex {
    const DESC: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4]
    };
}

pub trait Drawable {
    type Data;
    fn prepare(&self) -> Self::Data;
}

pub trait Draw<Item>
where
    Item: Drawable,
{
    fn draw(&mut self, items: &[Item]);
}


