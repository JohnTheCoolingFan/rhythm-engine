/// This entire module is a hack to make windows function as panels.
/// Tile based UI is difficult to express with ECS functions so this is the next best thing.
/// - Start with the maximal available realestate
/// - Split and subtract the area needed by the current widget
/// - Consume and pipe realestate through systems or allocate to resources used by each system
use crate::utils::*;
use tap::{Pipe, Tap};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Realestate {
    min_x: P32,
    min_y: P32,
    max_x: P32,
    max_y: P32,
}

impl Realestate {
    fn vsplit<const N: usize>(self, proportions: [P32; N]) -> [Self; N] {
        let (denom, available_x, mut scan_x) = (
            proportions.iter().sum::<P32>(),
            self.max_x - self.min_x,
            self.min_x,
        );

        proportions.map(|p| Self {
            min_y: self.min_y,
            max_y: self.max_y,
            min_x: scan_x,
            max_x: (scan_x + available_x * (p / denom))
                .raw()
                .clamp(self.min_x.raw(), self.max_x.raw())
                .pipe(p32)
                .tap(|new_max| scan_x = *new_max),
        })
    }

    fn hsplit<const N: usize>(self, proportions: [P32; N]) -> [Self; N] {
        let (denom, available_y, mut scan_y) = (
            proportions.iter().sum::<P32>(),
            self.max_y - self.min_y,
            self.min_y,
        );

        proportions.map(|p| Self {
            min_x: self.min_x,
            max_x: self.max_x,
            min_y: scan_y,
            max_y: (scan_y + available_y * (p / denom))
                .raw()
                .clamp(self.min_y.raw(), self.max_y.raw())
                .pipe(p32)
                .tap(|new_max| scan_y = *new_max),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use test_case::test_case;

    fn vertical_four_way_1080p() -> [Realestate; 4] {
        [
            Realestate {
                min_x: p32(0.0),
                min_y: p32(0.0),
                max_x: p32(480.0),
                max_y: p32(1080.0),
            },
            Realestate {
                min_x: p32(480.0),
                min_y: p32(0.0),
                max_x: p32(960.0),
                max_y: p32(1080.0),
            },
            Realestate {
                min_x: p32(960.0),
                min_y: p32(0.0),
                max_x: p32(1440.0),
                max_y: p32(1080.0),
            },
            Realestate {
                min_x: p32(1440.0),
                min_y: p32(0.0),
                max_x: p32(1920.0),
                max_y: p32(1080.0),
            },
        ]
    }

    fn horizontal_four_way_1080p() -> [Realestate; 4] {
        [
            Realestate {
                min_x: p32(0.0),
                min_y: p32(0.0),
                max_x: p32(1920.0),
                max_y: p32(270.0),
            },
            Realestate {
                min_x: p32(0.0),
                min_y: p32(270.0),
                max_x: p32(1920.0),
                max_y: p32(540.0),
            },
            Realestate {
                min_x: p32(0.0),
                min_y: p32(540.0),
                max_x: p32(1920.0),
                max_y: p32(810.0),
            },
            Realestate {
                min_x: p32(0.0),
                min_y: p32(810.0),
                max_x: p32(1920.0),
                max_y: p32(1080.0),
            },
        ]
    }

    #[test_case(
        Realestate {
            min_x: p32(0.),
            min_y: p32(0.),
            max_x: p32(1920.),
            max_y: p32(1080.),
        },
        [1., 1., 1., 1.].map(p32),
        &vertical_four_way_1080p();
        "from_zero"
    )]
    #[test_case(
        Realestate {
            min_x: p32(480.),
            min_y: p32(0.),
            max_x: p32(1920.),
            max_y: p32(1080.),
        },
        [1., 1., 1.].map(p32),
        &vertical_four_way_1080p()[1..];
        "from_non_zero"
    )]
    fn vsplit<const N: usize>(initial: Realestate, proportions: [P32; N], expected: &[Realestate]) {
        assert_eq!(initial.vsplit(proportions), expected);
    }

    #[test_case(
        Realestate {
            min_x: p32(0.),
            min_y: p32(0.),
            max_x: p32(1920.),
            max_y: p32(1080.),
        },
        [1., 1., 1., 1.].map(p32),
        &horizontal_four_way_1080p();
        "from_zero"
    )]
    #[test_case(
        Realestate {
            min_x: p32(0.0),
            min_y: p32(270.0),
            max_x: p32(1920.0),
            max_y: p32(1080.0),
        },
        [1., 1., 1.].map(p32),
        &horizontal_four_way_1080p()[1..];
        "from_non_zero"
    )]
    fn hsplit<const N: usize>(initial: Realestate, proportions: [P32; N], expected: &[Realestate]) {
        assert_eq!(initial.hsplit(proportions), expected);
    }
}
