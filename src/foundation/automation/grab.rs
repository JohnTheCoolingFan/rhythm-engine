use glam::Vec2;
use crate::foundation::automation::*;

pub struct Grab {
    direction: Automation,
    strength: Automation
}

pub struct GrabSeeker<'a> {
    dir_seeker: AutomationSeeker<'a>,
    str_seeker: AutomationSeeker<'a>,
    grab: &'a Grab
}

//apply with average or with sequential operations? :thonk:
impl<'a> GrabSeeker<'a> {
    pub fn apply(&self, last: Vec2, curr: Vec2) -> Vec2 {
    }
}

impl<'a> Seeker<'a> for GrabSeeker<'a> {
    fn seek
}
