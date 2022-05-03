use noisy_float::prelude::*;

pub trait Lerp {
    fn lerp(self, other: Self, t: N32) -> Self;
}

pub trait Quantify {
    fn quantify(&self) -> N32;
}

pub trait SliceExt<'a, T> {
    fn seek(self, by: impl Quantify) -> usize;
    fn first_before(self, t: N32) -> Option<&'a T>;
    fn first_after(self, t: N32) -> Option<&'a T>;
}

impl Quantify for N32 {
    fn quantify(&self) -> N32 {
        *self
    }
}

impl<'a, T: Quantify> SliceExt<'a, T> for &'a [T] {
    fn seek(self, to: impl Quantify) -> usize {
        let index = self
            .binary_search_by(|item| item.quantify().cmp(&to.quantify()))
            .unwrap_or_else(|index| match index {
                0 => 0,
                index if self.len() <= index => self.len() - 1,
                _ => index - 1,
            });

        let found = &self[index];

        let to_skip = self
            .iter()
            .skip(index + 1)
            .take_while(|item| found.quantify() == item.quantify())
            .count();

        index + to_skip
    }

    fn first_before(self, t: N32) -> Option<&'a T> {
        self.iter().take_while(|item| item.quantify() <= t).last()
    }

    fn first_after(self, t: N32) -> Option<&'a T> {
        self.iter()
            .rev()
            .take_while(|item| t < item.quantify())
            .last()
    }
}
