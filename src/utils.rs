use bevy::prelude::*;
use derive_more::{Deref, From};
use educe::*;
use itertools::Itertools;
use noisy_float::{prelude::*, FloatChecker, NoisyFloat};
use tap::{Pipe, Tap};

use std::marker::PhantomData;

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
        0. < (end.x - start.x) * (self.y - start.y) - (end.y - start.y) * (self.x - start.x)
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
    fn at_or_after(self, offset: P32) -> &'a [T];
    fn interp(self, offset: P32) -> Result<<T as Lerp>::Output, &'a T>
    where
        T: Lerp;
}

/// Must be non-empty and sorted
impl<'a, T: Quantify> ControlTable<'a, T> for &'a [T] {
    fn at_or_after(self, offset: P32) -> &'a [T] {
        self.iter()
            .take_while(|item| item.quantify() < offset)
            .count()
            .saturating_sub(1)
            .pipe(|start| &self[start..])
    }

    fn interp(self, offset: P32) -> Result<<T as Lerp>::Output, &'a T>
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

pub trait Vec2IterExt: Iterator<Item = Vec2> + Clone {
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

    fn centroid(self) -> Vec2 {
        match self.clone().count() {
            0 => Vec2::default(),
            n => self.sum::<Vec2>().pipe(|sum| Vec2 {
                x: sum.x / n as f32,
                y: sum.y / n as f32,
            }),
        }
    }
}

impl<T: Iterator<Item = Vec2> + Clone> Vec2IterExt for T {}

#[derive(Educe)]
#[educe(PartialEq, Ord, Eq, PartialOrd)]
#[derive(Component, Deref)]
pub struct GenID<T> {
    #[deref]
    id: Entity,
    #[educe(PartialEq(ignore), Ord(ignore), Eq(ignore), PartialOrd(ignore))]
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

pub trait Property<Target> {
    fn ensure(target: &mut Target);
}

#[derive(Deref, Clone, Copy)]
pub struct Ensured<T, P: Property<T>> {
    #[deref]
    data: T,
    _phantom: PhantomData<P>,
}

impl<T, P: Property<T>> Ensured<T, P> {
    pub fn new(mut data: T) -> Self {
        P::ensure(&mut data);
        Self {
            data,
            _phantom: PhantomData,
        }
    }

    pub fn apply(&mut self, func: impl Fn(&mut T)) {
        func(&mut self.data);
        P::ensure(&mut self.data);
    }
}

impl<T, P: Property<T>> From<T> for Ensured<T, P> {
    fn from(value: T) -> Self {
        Ensured::new(value)
    }
}

// First encountered duplicates will be dropped
//
//      0, 0, 1, 2, 2, 3
//      X        X
//
// Allows for contiguous unique storage with modifications via a `push_back`
// Good for structures with infrequent mutation and frequent reads
#[derive(Clone, Copy)]
pub struct FrontDupsDropped;

impl<T: PartialEq + Ord + Clone> Property<Vec<T>> for FrontDupsDropped {
    fn ensure(target: &mut Vec<T>) {
        *target = target
            .tap_mut(|target| target.sort()) // Must be stable!
            .iter()
            .cloned()
            .coalesce(|prev, curr| prev.eq(&curr).then_some(curr.clone()).ok_or((prev, curr)))
            .collect::<Vec<_>>()
    }
}
