use itertools::Itertools;
use noisy_float::prelude::*;
use std::cmp::Ordering;

pub trait Quantify {
    fn quantify(&self) -> N32;
}

pub trait Interpolate: Sized {
    type Output;
    fn interp(&self, previous: &[Self], t: N32) -> Self::Output;
}

pub trait SliceSeekExt<T> {
    fn seek(self, by: impl Quantify) -> usize;
}

pub trait SliceInterpExt<T: Interpolate> {
    fn interp(self, t: N32) -> Option<T::Output>;
}

impl Quantify for N32 {
    fn quantify(&self) -> N32 {
        *self
    }
}

impl<T: Quantify> SliceSeekExt<T> for &[T] {
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
}

impl<T: Quantify + Interpolate> SliceInterpExt<T> for &[T] {
    fn interp(self, t: N32) -> Option<T::Output> {
        let passed = self.iter().take_while(|item| item.quantify() < t).count();

        self.iter()
            .rev()
            .take_while(|item| t <= item.quantify())
            .last()
            .map(|curr| curr.interp(&self[..passed], t))
    }
}
