use crate::foundation::automation::automation::AutomationSeeker;
use crate::utils::*;
use crate::foundation::complex_spline::*;
use crate::foundation::automation::*;
use glam::Vec2;

use super::segment::SegmentSeeker;

struct ComplexSpline {
    curve: CurveChain,
    offset: f32,
    automation: Automation
}

impl ComplexSpline {
    pub fn new(start: f32, end: f32, intial: Ctrl) -> Self {
        assert!(start <= end, "end offset cannot be less than start");
        let mut new_curve = CurveChain::new();
        new_curve.push_from_absolute(intial);
        Self {
            curve: new_curve,
            offset: start,
            automation: Automation::new(0., 1., end - start, false)
        }
    }
}

struct CompSplSeeker<'a> {
    c_spline: &'a ComplexSpline,
    auto_seeker: AutomationSeeker<'a>,
    segment_seeker: SegmentSeeker<'a>
}

impl<'a> Seeker<Vec2> for CompSplSeeker<'a> {
    fn jump(&mut self, val: f32) -> Vec2 {
        let old_index = self.auto_seeker.get_index();
        let y = self.auto_seeker.jump(val - self.c_spline.offset);
        let new_index = self.auto_seeker.get_index();
        if old_index != new_index {
            self.segment_seeker = self.c_spline.curve[new_index].seeker();
        }
        self.segment_seeker.seek(y)
    }

    fn seek(&mut self, val: f32) -> Vec2 {
        let old_index = self.auto_seeker.get_index();
        let y = self.auto_seeker.seek(val - self.c_spline.offset);
        let new_index = self.auto_seeker.get_index();
        if old_index != new_index {
            self.segment_seeker = self.c_spline.curve[new_index].seeker();
        }
        self.segment_seeker.seek(y)
    }
}

impl<'a> Seekable<'a, Vec2> for ComplexSpline {
    type Seeker = CompSplSeeker<'a>;
    fn seeker(&'a self) -> Self::Seeker {
        CompSplSeeker {
            auto_seeker: self.automation.seeker(),
            segment_seeker: self.curve[0].seeker(),
            c_spline: &self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ggez::{
        graphics::*,
        event::{self, EventHandler, KeyCode, KeyMods, MouseButton},
        Context,
        GameResult
    };

    struct CompSplTest {
        cps: ComplexSpline
    }

    /*impl CompSplTest {
        fn new() -> Self {
            Self {
                cps: ComplexSpline::new(0, , intial)
            }
        }
    }*/
}
