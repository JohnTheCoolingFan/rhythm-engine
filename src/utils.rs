use bevy::prelude::*;
use itertools::Itertools;
use noisy_float::{prelude::*, FloatChecker, NoisyFloat};
use tap::Pipe;

#[derive(Debug, Clone, Copy)]
pub struct UnitIntervalChecker;

impl FloatChecker<f32> for UnitIntervalChecker {
    #[inline]
    fn check(value: f32) -> bool {
        (0.0..=1.0).contains(&value)
    }

    #[inline]
    fn assert(value: f32) {
        debug_assert!(Self::check(value), "Unexpected non-unit float: {value}");
    }
}

pub type T32 = NoisyFloat<f32, UnitIntervalChecker>;

pub fn t32(value: f32) -> T32 {
    T32::new(value)
}

#[derive(Debug, Clone, Copy)]
pub struct PositiveFloatChecker;

impl FloatChecker<f32> for PositiveFloatChecker {
    #[inline]
    fn check(value: f32) -> bool {
        (0.0..).contains(&value)
    }

    #[inline]
    fn assert(value: f32) {
        debug_assert!(Self::check(value), "Unexpected non-positive float: {value}");
    }
}

pub type P32 = NoisyFloat<f32, PositiveFloatChecker>;

pub fn p32(value: f32) -> P32 {
    P32::new(value)
}

pub trait Quantify {
    fn quantify(&self) -> P32;
}

pub trait Lerp {
    type Output;
    fn lerp(&self, next: &Self, t: T32) -> Self::Output;
}

pub trait CompletionRatio {
    fn completion_ratio(self, start: Self, end: Self) -> T32;
}

pub trait Vec2Ext {
    fn is_left(&self, start: &Self, end: &Self) -> bool;
    fn rotate_about(&self, vec: Self, theta: R32) -> Self;
}

impl Quantify for P32 {
    fn quantify(&self) -> P32 {
        *self
    }
}

impl<Checker: FloatChecker<f32>> Lerp for NoisyFloat<f32, Checker> {
    type Output = Self;
    fn lerp(&self, next: &Self, t: T32) -> Self::Output {
        Self::new(self.raw() + (next.raw() - self.raw()) * t.raw())
    }
}

impl<Checker: FloatChecker<f32>> CompletionRatio for NoisyFloat<f32, Checker> {
    fn completion_ratio(self, start: Self, end: Self) -> T32 {
        t32((self.raw() - start.raw()) / (end.raw() - start.raw()))
    }
}

impl Vec2Ext for Vec2 {
    fn is_left(&self, start: &Self, end: &Self) -> bool {
        0. < ((end.x - start.x) * (self.y - start.y) - (end.y - start.y) * (self.x - start.x))
    }

    fn rotate_about(&self, vec: Self, radians: R32) -> Self {
        let c = radians.raw().cos();
        let s = radians.raw().sin();

        Self::new(
            c * (self.x - vec.x) - s * (self.y - vec.y) + vec.x,
            s * (self.x - vec.x) + c * (self.y - vec.y) + vec.y,
        )
    }
}

pub trait ControlTable<'a, T> {
    fn seek(self, to: impl Quantify) -> usize;
    fn can_skip_reindex(self, offset: P32) -> bool;
    fn reindex_through(self, offset: P32, old: usize) -> usize;
    fn interp(self, offset: P32) -> Result<<T as Lerp>::Output, &'a T>
    where
        T: Lerp;
}

/// Must be non-empty and sorted
impl<'a, T: Quantify> ControlTable<'a, T> for &'a [T] {
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

    fn can_skip_reindex(self, offset: P32) -> bool {
        self.last().map_or(true, |item| item.quantify() < offset)
    }

    #[rustfmt::skip]
    fn reindex_through(self, offset: P32, old: usize) -> usize {
        self.iter()
            .enumerate()
            .skip(old)
            .coalesce(|prev, curr| (prev.1.quantify() == curr.1.quantify())
                .then(|| curr)
                .ok_or((prev, curr))
            )
            .take(4)
            .take_while(|(_, item)| item.quantify() < offset)
            .last()
            .map(|(index, _)| index)
            .unwrap_or_else(|| self.seek(offset))
    }

    fn interp(self, offset: P32) -> Result<<T as Lerp>::Output, &'a T>
    where
        T: Lerp,
    {
        let start = self
            .iter()
            .take_while(|item| item.quantify() < offset)
            .count()
            .saturating_sub(1);

        match &self[start..] {
            [prev, curr, ..] => offset
                .completion_ratio(prev.quantify(), curr.quantify())
                .pipe(|t| prev.lerp(curr, t))
                .pipe(Ok),
            _ => Err(self.last().unwrap()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    CounterClockWise,
    CoLinear,
    ClockWise,
}

pub trait OrientationExt: Iterator<Item = Vec2> + Clone {
    fn orientation(self) -> Orientation {
        match self
            .clone()
            .chain(self.take(1))
            .tuple_windows::<(_, _)>()
            .map(|(p, q)| (q.x - p.x) * (q.y + p.y))
            .sum::<f32>()
        {
            sum if sum.abs() <= f32::EPSILON => Orientation::CoLinear,
            sum if sum < 0. => Orientation::CounterClockWise,
            sum if 0. < sum => Orientation::ClockWise,
            _ => unreachable!(),
        }
    }
}

impl<T: Iterator<Item = Vec2> + Clone> OrientationExt for T {}
