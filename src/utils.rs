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

pub trait UnitIntervalExt {
    fn inv(self) -> Self;
}

impl UnitIntervalExt for T32 {
    fn inv(self) -> Self {
        t32(1. - self.raw())
    }
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
    type Output;
    fn lerp(&self, other: &Self, t: T32) -> Self::Output;
}

/// Requires underlying dataset to be sorted
/// Dataset should be small (< 0.5KB) and trivially linear searchable
pub trait SliceExt<'a, T> {
    fn seek(self, by: impl Quantify) -> usize;
    fn before_or_at(self, offset: R32) -> &'a [T];
    fn after(self, offset: R32) -> &'a [T];
    fn interp(self, offset: R32) -> Option<<T as Lerp>::Output>
    where
        T: Lerp;
    fn interp_or_last(self, offset: R32) -> <T as Lerp>::Output
    where
        T: Lerp;
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
        println!("{} {}", first, second);
        t32(((self - first) / (second - first)).raw())
    }
}

impl Quantify for R32 {
    fn quantify(&self) -> R32 {
        *self
    }
}

impl Lerp for R32 {
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

    fn before_or_at(self, offset: R32) -> Self {
        let end = self
            .iter()
            .take_while(|item| item.quantify() <= offset)
            .count();

        &self[..end]
    }

    fn after(self, offset: R32) -> Self {
        let keep_size = self
            .iter()
            .rev()
            .take_while(|item| offset < item.quantify())
            .count();

        &self[self.len() - keep_size..]
    }

    fn interp(self, offset: R32) -> Option<<T as Lerp>::Output>
    where
        T: Lerp,
    {
        let (follow, control) = (self.before_or_at(offset).last(), self.after(offset).first());

        follow.zip(control).map(|(follow, control)| {
            control.lerp(
                follow,
                offset.unit_interval(follow.quantify(), control.quantify()),
            )
        })
    }

    fn interp_or_last(self, offset: R32) -> <T as Lerp>::Output
    where
        T: Lerp,
    {
        self.interp(offset).unwrap_or_else(|| {
            let last = self.last().unwrap();
            last.lerp(last, t32(0.))
        })
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
    fn slice_ext_before_or_at() {
        assert_eq!(numbers().before_or_at(r32(0.0)), [] as [R32; 0]);
        assert_eq!(numbers().before_or_at(r32(2.0)), &[r32(1.0), r32(2.0)]);
    }

    #[test]
    fn slice_ext_after() {
        assert_eq!(numbers().after(r32(10.)), [] as [R32; 0]);
        assert_eq!(numbers().after(r32(7.5)), &[r32(8.0), r32(9.0), r32(10.0)]);
    }
}
