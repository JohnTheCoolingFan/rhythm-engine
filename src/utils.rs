use noisy_float::prelude::*;

pub trait Quantify {
    fn quantify(&self) -> N32;
}

pub trait FloatExt {
    fn quant_floor(self, period: Self, offset: Self) -> Self;
}

pub trait Lerp {
    fn lerp(self, other: Self, t: N32) -> Self;
}

pub trait SliceExt<'a, T> {
    fn seek(self, by: impl Quantify) -> usize;
    fn first_before(self, t: N32) -> Option<&'a T>;
    fn first_before_or_at(self, t: N32) -> Option<&'a T>;
    fn first_after(self, t: N32) -> Option<&'a T>;
    fn first_after_or_at(self, t: N32) -> Option<&'a T>;
}

impl Quantify for N32 {
    fn quantify(&self) -> N32 {
        *self
    }
}

impl FloatExt for N32 {
    fn quant_floor(self, period: Self, offset: Self) -> Self {
        if f32::EPSILON < period.raw().abs() {
            ((self - offset) / period).floor() * period + offset
        } else {
            self
        }
    }
}

impl Lertp for N32 {
    fn lerp(self, other: Self, t: Self, amount: Self) -> Self {
        self + (other - self) * amount
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
        self.iter().take_while(|item| item.quantify() < t).last()
    }

    fn first_before_or_at(self, t: N32) -> Option<&'a T> {
        self.iter().take_while(|item| item.quantify() <= t).last()
    }

    fn first_after(self, t: N32) -> Option<&'a T> {
        self.iter()
            .rev()
            .take_while(|item| t < item.quantify())
            .last()
    }

    fn first_after_or_at(self, t: N32) -> Option<&'a T> {
        self.iter()
            .rev()
            .take_while(|item| t <= item.quantify())
            .last()
    }
}
