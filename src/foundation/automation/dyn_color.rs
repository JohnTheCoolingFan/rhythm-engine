use crate::foundation::automation::*;
use ggez::graphics::Color;

struct ColorSegment {
    color: Color,
    offset: f32
}

pub struct DynColor {
    top_colors: Vec<ColorSegment>,
    automation: Automation,
    bottom_colors: Vec<ColorSegment>
}

impl DynColor {
    pub fn new(len: f32) -> Self {
        Self {
            top_colors: vec![ColorSegment{ color: Color::WHITE, offset: 0. }],
            automation: Automation::new(0., 1., len, false),
            bottom_colors: vec![ColorSegment{ color: Color::BLACK, offset: 0. }]
        }
    }
}

