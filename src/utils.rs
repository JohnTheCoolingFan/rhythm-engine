use bevy::{math::f64::DVec2, prelude::*};
use derive_more::{Deref, From};
use itertools::Itertools;
use noisy_float::{prelude::*, FloatChecker, NoisyFloat};
use tap::Pipe;

use std::marker::PhantomData;

#[derive(Debug, Clone, Copy)]
pub struct UnitIntervalChecker;

impl FloatChecker<f64> for UnitIntervalChecker {
    #[inline]
    fn check(value: f64) -> bool {
        (0.0..=1.0).contains(&value)
    }

    #[inline]
    fn assert(value: f64) {
        debug_assert!(Self::check(value), "Unexpected non-unit float: {value}");
    }
}

pub type T64 = NoisyFloat<f64, UnitIntervalChecker>;

pub fn t64(value: f64) -> T64 {
    T64::new(value)
}

#[derive(Debug, Clone, Copy)]
pub struct PositiveFloatChecker;

impl FloatChecker<f64> for PositiveFloatChecker {
    #[inline]
    fn check(value: f64) -> bool {
        (0.0..).contains(&value)
    }

    #[inline]
    fn assert(value: f64) {
        debug_assert!(Self::check(value), "Unexpected non-positive float: {value}");
    }
}

pub type P64 = NoisyFloat<f64, PositiveFloatChecker>;

pub fn p64(value: f64) -> P64 {
    P64::new(value)
}

pub trait Quantify {
    fn quantify(&self) -> P64;
}

pub trait Lerp {
    type Output;
    fn lerp(&self, next: &Self, t: T64) -> Self::Output;
}

pub trait CompletionRatio {
    fn completion_ratio(self, start: Self, end: Self) -> T64;
}

pub trait Vec2Ext {
    fn is_left(&self, start: &Self, end: &Self) -> bool;
    fn rotate_about(&self, vec: Self, theta: R64) -> Self;
}

impl Quantify for P64 {
    fn quantify(&self) -> P64 {
        *self
    }
}

impl<Checker: FloatChecker<f64>> Lerp for NoisyFloat<f64, Checker> {
    type Output = Self;
    fn lerp(&self, next: &Self, t: T64) -> Self::Output {
        Self::new(self.raw() + (next.raw() - self.raw()) * t.raw())
    }
}

impl<Checker: FloatChecker<f64>> CompletionRatio for NoisyFloat<f64, Checker> {
    fn completion_ratio(self, start: Self, end: Self) -> T64 {
        t64((self.raw() - start.raw()) / (end.raw() - start.raw()))
    }
}

impl Vec2Ext for DVec2 {
    fn is_left(&self, start: &Self, end: &Self) -> bool {
        0. < (end.x - start.x) * (self.y - start.y) - (end.y - start.y) * (self.x - start.x)
    }

    fn rotate_about(&self, vec: Self, radians: R64) -> Self {
        let c = radians.raw().cos();
        let s = radians.raw().sin();

        Self::new(
            c * (self.x - vec.x) - s * (self.y - vec.y) + vec.x,
            s * (self.x - vec.x) + c * (self.y - vec.y) + vec.y,
        )
    }
}

pub trait ControlTable<'a, T> {
    fn at_or_after(self, offset: P64) -> &'a [T];
    fn interp(self, offset: P64) -> Result<<T as Lerp>::Output, &'a T>
    where
        T: Lerp;
}

/// Must be non-empty and sorted
impl<'a, T: Quantify> ControlTable<'a, T> for &'a [T] {
    fn at_or_after(self, offset: P64) -> &'a [T] {
        self.iter()
            .take_while(|item| item.quantify() < offset)
            .count()
            .saturating_sub(1)
            .pipe(|start| &self[start..])
    }

    fn interp(self, offset: P64) -> Result<<T as Lerp>::Output, &'a T>
    where
        T: Lerp,
    {
        match self.at_or_after(offset) {
            [prev, curr, ..] => offset
                .completion_ratio(prev.quantify(), curr.quantify())
                .pipe(|t| prev.lerp(curr, t))
                .pipe(Ok),
            [single] => Err(single),
            _ => panic!("Unexpected existing no item control table"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    CounterClockWise,
    CoLinear,
    ClockWise,
}

pub trait OrientationExt: Iterator<Item = DVec2> + Clone {
    fn orientation(self) -> Orientation {
        match self
            .clone()
            .chain(self.take(1))
            .tuple_windows::<(_, _)>()
            .map(|(p, q)| (q.x - p.x) * (q.y + p.y))
            .sum::<f64>()
        {
            sum if sum.abs() <= f64::EPSILON => Orientation::CoLinear,
            sum if sum < 0. => Orientation::CounterClockWise,
            sum if 0. < sum => Orientation::ClockWise,
            _ => unreachable!(),
        }
    }
}

#[derive(Component, Deref)]
pub struct GenID<T> {
    #[deref]
    id: Entity,
    _phantom: PhantomData<T>,
}

#[rustfmt::skip]
impl<T> GenID<T> {
    pub fn new(id: Entity) -> Self {
        Self { id, _phantom: PhantomData }
    }
}

#[rustfmt::skip]
impl<T> Clone for GenID<T> {
    fn clone(&self) -> Self {
        Self { id: self.id, _phantom: PhantomData }
    }
}

impl<T> Copy for GenID<T> {}

impl<T: Iterator<Item = DVec2> + Clone> OrientationExt for T {}

pub const MAX_CHANNELS: usize = 256;

#[derive(Deref, DerefMut, From, Clone, Copy, Resource)]
pub struct Table<T>(pub [T; MAX_CHANNELS]);

impl<T> Table<T> {
    pub fn fill_with(&mut self, func: impl Fn() -> T) {
        self.0 = [(); MAX_CHANNELS].map(|_| func());
    }
}

impl<T: Default> Default for Table<T> {
    fn default() -> Self {
        Self([(); MAX_CHANNELS].map(|_| T::default()))
    }
}
