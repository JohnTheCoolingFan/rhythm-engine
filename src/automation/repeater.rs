use crate::automation::*;

struct RepeaterClamp {
    start: T32,
    end: T32,
    weight: Weight,
}

impl RepeaterClamp {
    fn eval(&self, t: T32) -> T32 {
        self.start.lerp(&self.end, self.weight.eval(t))
    }
}

#[derive(Component)]
pub struct Repeater {
    duration: R32,
    ceil: RepeaterClamp,
    floor: RepeaterClamp,
    repeat_bounds: bool,
}
