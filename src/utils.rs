use noisy_float::prelude::*;

pub trait Quantify {
    fn quantify(&self) -> N32;
}

pub trait FloatExt {
    fn quant_floor(self, period: Self, offset: Self) -> Self;
}

pub trait Lerp {
    fn lerp(&self, other: &Self, t: N32) -> Self;
}

pub trait Sample: Sized + Clone {
    fn sample(&self, _other: &Self, _t: N32) -> Self {
        self.clone()
    }
}

/// Requires underlying data to be sorted
pub trait SliceExt<'a, T> {
    fn seek(self, by: impl Quantify) -> usize;
    /// Should be small and trivially linear searchable
    fn before(self, t: N32) -> &'a [T];
    /// Should be small and trivially linear searchable
    fn before_or_at(self, t: N32) -> &'a [T];
    /// Should be small and trivially linear searchable
    fn after(self, t: N32) -> &'a [T];
    /// Should be small and trivially linear searchable
    fn after_or_at(self, t: N32) -> &'a [T];
    /// Should be small and trivially linear searchable
    fn sample(self, t: N32) -> Option<T>
    where
        T: Sample;
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

impl Lerp for N32 {
    fn lerp(&self, other: &Self, amount: N32) -> Self {
        *self + (*other - *self) * amount
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

    fn before(self, t: N32) -> &'a [T] {
        &self[..self.iter().take_while(|item| item.quantify() < t).count()]
    }

    fn before_or_at(self, t: N32) -> &'a [T] {
        &self[..self.iter().take_while(|item| item.quantify() <= t).count()]
    }

    fn after(self, t: N32) -> &'a [T] {
        let num = self
            .iter()
            .rev()
            .take_while(|item| t < item.quantify())
            .count();

        &self[self.len() - num..]
    }

    fn after_or_at(self, t: N32) -> &'a [T] {
        let num = self
            .iter()
            .rev()
            .take_while(|item| t <= item.quantify())
            .count();

        &self[self.len() - num..]
    }

    fn sample(self, t: N32) -> Option<T>
    where
        T: Sample,
    {
        self.before_or_at(t).last().and_then(|control| {
            self.after(t)
                .first()
                .map(|follow| control.sample(&follow, t))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn numbers() -> Vec<N32> {
        [1., 2., 3., 4., 5., 6., 7., 7., 8., 9., 10.]
            .into_iter()
            .map(n32)
            .collect::<Vec<_>>()
    }

    #[test]
    fn slice_ext_before() {
        assert_eq!(numbers().before(n32(0.0)), [] as [N32; 0]);
        assert_eq!(numbers().before(n32(2.0)), &[n32(1.0)]);
    }

    #[test]
    fn slice_ext_before_or_at() {
        assert_eq!(numbers().before_or_at(n32(0.0)), [] as [N32; 0]);
        assert_eq!(numbers().before_or_at(n32(2.0)), &[n32(1.0), n32(2.0)]);
    }

    #[test]
    fn slice_ext_after() {
        assert_eq!(numbers().after(n32(10.)), [] as [N32; 0]);
        assert_eq!(numbers().after(n32(7.5)), &[n32(8.0), n32(9.0), n32(10.0)]);
    }

    #[test]
    fn slice_ext_after_or_at() {
        assert_eq!(numbers().after_or_at(n32(10.1)), [] as [N32; 0]);
        assert_eq!(
            numbers().after_or_at(n32(7.)),
            &[n32(7.0), n32(7.0), n32(8.0), n32(9.0), n32(10.0)]
        );
    }
}
