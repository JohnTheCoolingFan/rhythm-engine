use noisy_float::prelude::*;
use std::cmp::Ordering;
use std::convert::identity;

pub trait Quantify {
    fn quantify(&self) -> N32;
}

pub trait Interpolate {
    type Output;
    fn interp(&self, other: &Self, t: N32) -> Self::Output;
    fn default(&self) -> Self::Output;
}

pub trait SliceSeekExt<T> {
    fn seek(self, by: impl FnMut(&T) -> Ordering) -> usize;
}

pub trait SliceInterpExt<T: Interpolate> {
    fn interp(self, t: N32) -> T::Output;
}

impl<T> SliceSeekExt<T> for &[T] {
    fn seek(self, by: impl FnMut(&T) -> Ordering) -> usize {
        self.binary_search_by(by)
            .map_err(|index| match index {
                0 => 0,
                index if self.len() <= index => self.len() - 1,
                _ => index - 1,
            })
            .unwrap_or_else(identity)
    }
}

impl<T: Quantify + Interpolate> SliceInterpExt<T> for &[T] {
    fn interp(self, t: N32) -> T::Output {
        match self
            .iter()
            .skip_while(|item| item.quantify() < t)
            .take(2)
            .collect::<Vec<_>>()
            .as_slice()
        {
            [first, second] => first.interp(second, t - first.quantify()),
            [last_item] => last_item.default(),
            _ => unreachable!(),
        }
    }
}
