/// This entire module is a hack to make windows function as panels.
/// Tile based UI is difficult to express with ECS functions so this is the next best thing.
/// - Start with the maximal available realestate
/// - Split and subtract the area needed by the current widget
/// - Consume and pipe realestate through systems
use crate::utils::*;

struct Realestate {}

impl Realestate {
    fn hsplit(self, t: P32, b: P32) -> (Self, Self) {
        todo!()
    }

    fn vsplit(self, l: P32, r: P32) -> (Self, Self) {
        todo!()
    }
}
