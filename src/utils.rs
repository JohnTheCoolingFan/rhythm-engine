use noisy_float::{prelude::*, FloatChecker, NoisyFloat};

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

pub trait Quantify {
    fn quantify(&self) -> R32;
}

pub trait FloatExt {
    fn quantized_floor(self, period: Self, offset: Self) -> Self;
    fn quantized_remainder(self, period: Self, offset: Self) -> Self;
    fn unit_interval(self, control: Self, follow: Self) -> T32;
}

pub trait Lerp {
    fn lerp(&self, other: &Self, t: T32) -> Self;
}

pub trait Sample: Sized + Clone {
    fn sample(&self, _other: &Self, _t: T32) -> Self {
        self.clone()
    }
}

/// Requires underlying data to be sorted
pub trait SliceExt<'a, T> {
    fn seek(self, by: impl Quantify) -> usize;
    /// Should be small and trivially linear searchable
    fn before(self, offset: R32) -> &'a [T];
    /// Should be small and trivially linear searchable
    fn before_or_at(self, offset: R32) -> &'a [T];
    /// Should be small and trivially linear searchable
    fn after(self, offset: R32) -> &'a [T];
    /// Should be small and trivially linear searchable
    fn after_or_at(self, offset: R32) -> &'a [T];
}

impl Quantify for R32 {
    fn quantify(&self) -> R32 {
        *self
    }
}

impl FloatExt for R32 {
    fn quantized_floor(self, period: Self, offset: Self) -> Self {
        if f32::EPSILON < period.raw().abs() {
            ((self - offset) / period).floor() * period + offset
        } else {
            self
        }
    }

    fn quantized_remainder(self, period: Self, offset: Self) -> Self {
        if f32::EPSILON < period.raw().abs() {
            (self - offset) % period
        } else {
            self
        }
    }

    fn unit_interval(self, first: Self, second: Self) -> T32 {
        t32(((self - first) / (second - first)).raw())
    }
}

impl Lerp for R32 {
    fn lerp(&self, other: &Self, t: T32) -> Self {
        *self + (*other - *self) * t.raw()
    }
}

impl Lerp for T32 {
    fn lerp(&self, other: &Self, t: T32) -> Self {
        *self + (*other - *self) * t.raw()
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

    fn before(self, offset: R32) -> &'a [T] {
        let end = self
            .iter()
            .take_while(|item| item.quantify() < offset)
            .count();

        &self[..end]
    }

    fn before_or_at(self, offset: R32) -> &'a [T] {
        let end = self
            .iter()
            .take_while(|item| item.quantify() <= offset)
            .count();

        &self[..end]
    }

    fn after(self, offset: R32) -> &'a [T] {
        let num = self
            .iter()
            .rev()
            .take_while(|item| offset < item.quantify())
            .count();

        &self[self.len() - num..]
    }

    fn after_or_at(self, offset: R32) -> &'a [T] {
        let num = self
            .iter()
            .rev()
            .take_while(|item| offset <= item.quantify())
            .count();

        &self[self.len() - num..]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn numbers() -> Vec<R32> {
        [1., 2., 3., 4., 5., 6., 7., 7., 8., 9., 10.]
            .into_iter()
            .map(r32)
            .collect::<Vec<_>>()
    }

    #[test]
    fn slice_ext_before() {
        assert_eq!(numbers().before(r32(0.0)), [] as [R32; 0]);
        assert_eq!(numbers().before(r32(2.0)), &[r32(1.0)]);
    }

    #[test]
    fn slice_ext_before_or_at() {
        assert_eq!(numbers().before_or_at(r32(0.0)), [] as [R32; 0]);
        assert_eq!(numbers().before_or_at(r32(2.0)), &[r32(1.0), r32(2.0)]);
    }

    #[test]
    fn slice_ext_after() {
        assert_eq!(numbers().after(r32(10.)), [] as [R32; 0]);
        assert_eq!(numbers().after(r32(7.5)), &[r32(8.0), r32(9.0), r32(10.0)]);
    }

    #[test]
    fn slice_ext_after_or_at() {
        assert_eq!(numbers().after_or_at(r32(10.1)), [] as [R32; 0]);
        assert_eq!(
            numbers().after_or_at(r32(7.)),
            &[r32(7.0), r32(7.0), r32(8.0), r32(9.0), r32(10.0)]
        );
    }
}
