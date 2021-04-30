use glam::Vec2;

use crate::core::automation;
use crate::core::complex_spline::ComplexSpline;

pub enum PolyEntityActiveMode {
    Hit(f32, f32),
    Hold(f32, f32, f32),
    Avoid(f32, f32),
}

pub struct PolyEntity {
    offsets: [f32; 2],
    hull: Vec<Vec2>,

    rotation: f32,
    scale: f32,

    active_mode: Option<PolyEntityActiveMode>,
    grab: f32,
    grab_dir: Vec2,

    vrtx_ctrls: Vec<ComplexSpline>,
    ctrl_routes: Vec<u32>,
}

impl Default for PolyEntity {
    fn default() -> PolyEntity {
        PolyEntity {
            offsets: [0.0; 2],
            hull: vec![Vec2::new(0.0, 0.0); 3],
            rotation: 0.0,
            scale: 1.0,
            active_mode: None,
            grab: 0.0,
            grab_dir: Vec2::new(0.0, 0.0),
            vrtx_ctrls: vec![],
            ctrl_routes: vec![],
        }
    }
}

impl PolyEntity {
    fn is_left(&self, a: usize, b: usize, point: &Vec2) -> bool {
        let line_start = self.hull[a];
        let line_end = self.hull[b];
        let segment = line_start - line_end;
        (segment.x * (point.y - line_end.y) - segment.y * (point.x - line_end.x)) > 0.0
    }

    fn in_triangle(&self, indices: &[usize; 3], point: &Vec2) -> bool {
        let a = self.is_left(indices[0], indices[1], point);

        a == self.is_left(indices[1], indices[2], point)
            && a == self.is_left(indices[2], indices[0], point)
    }
}
