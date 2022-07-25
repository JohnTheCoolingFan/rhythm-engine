use bevy::prelude::*;
use itertools::Itertools;
use noisy_float::{prelude::*, FloatChecker, NoisyFloat};

#[derive(Debug, Clone, Copy)]
pub struct UnitIntervalChecker;

impl FloatChecker<f32> for UnitIntervalChecker {
    fn check(value: f32) -> bool {
        (0.0..=1.0).contains(&value)
    }

    fn assert(value: f32) {
        debug_assert!(
            Self::check(value),
            "Expected within 0.0 - 1.0 (inclusive). Actual value: {value}"
        );
    }
}

pub type T32 = NoisyFloat<f32, UnitIntervalChecker>;

pub fn t32(value: f32) -> T32 {
    T32::new(value)
}

#[derive(Debug, Clone, Copy)]
pub struct PositiveFloatChecker;

impl FloatChecker<f32> for PositiveFloatChecker {
    fn check(value: f32) -> bool {
        (0.0..).contains(&value)
    }

    fn assert(value: f32) {
        debug_assert!(
            Self::check(value),
            "Expected positive float. Actual value: {value}"
        );
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

pub trait FloatExt {
    fn unit_interval(self, befor: Self, after: Self) -> T32;
}

pub trait Vec2Ext {
    fn is_left(&self, start: &Self, end: &Self) -> bool;
    /// Returns vec rotated about self
    fn rotate(&self, vec: &Self, theta: R32) -> Self;
}

pub trait MatExt {
    fn into_matrix(self) -> Mat3;
}

impl Quantify for P32 {
    fn quantify(&self) -> P32 {
        *self
    }
}

impl Lerp for R32 {
    type Output = Self;
    fn lerp(&self, other: &Self, t: T32) -> Self::Output {
        *self + (*other - *self) * t.raw()
    }
}

impl Lerp for P32 {
    type Output = Self;
    fn lerp(&self, other: &Self, t: T32) -> Self::Output {
        *self + (*other - *self) * t.raw()
    }
}

impl Lerp for T32 {
    type Output = Self;
    fn lerp(&self, other: &Self, t: T32) -> Self::Output {
        *self + (other.raw() - self.raw()) * t.raw()
    }
}

impl FloatExt for P32 {
    fn unit_interval(self, before: Self, after: Self) -> T32 {
        t32(((self - before) / (after - before)).raw())
    }
}

impl Vec2Ext for Vec2 {
    fn is_left(&self, start: &Self, end: &Self) -> bool {
        ((end.x - start.x) * (self.y - start.y) - (end.y - start.y) * (self.x - start.x)) > 0.
    }

    fn rotate(&self, vec: &Self, theta: R32) -> Self {
        let c = theta.raw().to_radians().cos();
        let s = theta.raw().to_radians().sin();

        Self::new(
            c * (vec.x - self.x) - s * (vec.y - self.y) + self.x,
            s * (vec.x - self.x) + c * (vec.y - self.y) + self.y,
        )
    }
}

impl MatExt for [[f32; 3]; 3] {
    fn into_matrix(self) -> Mat3 {
        Mat3::from_cols_array_2d(&self).transpose()
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
            .take_while(|item| offset < item.quantify())
            .count();

        match &self[start..] {
            [prev, curr, ..] => {
                Ok(prev.lerp(curr, offset.unit_interval(prev.quantify(), curr.quantify())))
            }
            _ => Err(self.last().unwrap()),
        }
    }
}

#[derive(PartialEq, Eq)]
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
            sum if sum < 0. => Orientation::CounterClockWise,
            sum if sum == 0. => Orientation::CoLinear,
            sum if 0. < sum => Orientation::ClockWise,
            _ => unreachable!(),
        }
    }
}

impl<T: Iterator<Item = Vec2> + Clone> OrientationExt for T {}
