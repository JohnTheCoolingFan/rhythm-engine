use crate::{foundation::automation::*, utils::Seeker};
use ggez::graphics::Color;

struct ColorSegment {
    color: Color,
    offset: f32
}

pub struct DynColor {
    upper_colors: Vec<ColorSegment>,
    automation: Automation,
    lower_colors: Vec<ColorSegment>
}

impl DynColor {
    pub fn new(len: f32) -> Self {
        Self {
            upper_colors: vec![ColorSegment{ color: Color::WHITE, offset: 0. }],
            automation: Automation::new(0., 1., len, false),
            lower_colors: vec![ColorSegment{ color: Color::BLACK, offset: 0. }]
        }
    }
}

pub struct DynColorSeeker<'a> {
    upper_index: usize,
    lower_index: usize,
    automation_seeker: automation::AutomationSeeker<'a>,
    dyncolor: &'a DynColor
}

impl<'a> DynColorSeeker<'a> {
    fn interp(&self, t: f32) -> Color {
        let c1 = self.dyncolor.lower_colors[self.lower_index].color;
        let c2 = self.dyncolor.upper_colors[self.upper_index].color;

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
            if offset <= self.dyncolor.upper_colors[self.upper_index].offset { break; }
            self.upper_index += 1;
        }
        while self.lower_index < self.dyncolor.lower_colors.len() {
            if offset <= self.dyncolor.lower_colors[self.lower_index].offset { break; }
            self.lower_index += 1;
        }
        let y = self.automation_seeker.seek(offset);
        self.interp(y)
    }

    fn jump(&mut self, val: f32) -> Color {
        self.upper_index = match self.dyncolor
            .upper_colors
            .binary_search_by(|anch| anch.offset.partial_cmp(&val).unwrap()) {
                Ok(index) => index,
                Err(index) => index
            };
        
        self.lower_index = match self.dyncolor
            .lower_colors
            .binary_search_by(|anch| anch.offset.partial_cmp(&val).unwrap()) {
                Ok(index) => index,
                Err(index) => index
            };

        let y = self.automation_seeker.seek(val);
        self.interp(y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ggez::{
        event::{self, EventHandler, KeyCode, KeyMods, MouseButton},
        graphics::MeshBuilder,
    };
    use ggez::{Context, GameResult};
    use crate::utils::from_end::FromEnd;

    struct DynColorTest {
        col: DynColor,
        dimensions: Vec2
    }

    impl DynColorTest {
        fn new() -> GameResult {
            Ok(Self {
                col: DynColor::new(2800.),
                dimensions: Vec(2800., 1100.)
            })
        }
    }


}
