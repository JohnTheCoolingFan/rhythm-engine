use bevy::prelude::*;
use bevy_egui::egui;
use derive_more::{Deref, From};
use educe::*;
use itertools::Itertools;
use lyon::tessellation::*;
use noisy_float::{prelude::*, FloatChecker, NoisyFloat};
use tap::{Pipe, Tap};

use std::{collections::HashSet, hash::Hash, marker::PhantomData};

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
    #[must_use]
    fn rotate_about(&self, vec: Self, theta: R32) -> Self;
    #[must_use]
    fn scale_about(&self, vec: Self, factor: R32) -> Self;
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

    fn scale_about(&self, vec: Self, factor: R32) -> Self {
        (*self - vec) * factor.raw() + vec
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

impl<T> From<Entity> for GenID<T> {
    fn from(value: Entity) -> Self {
        Self {
            id: value,
            _phantom: PhantomData,
        }
    }
}

impl<T> Clone for GenID<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            _phantom: PhantomData,
        }
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

#[derive(Deref, Clone, Copy, Debug)]
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

#[derive(Clone, Copy, Debug)]
pub struct StableDeduped;

impl<T: PartialEq + Ord + Clone + Hash> Property<Vec<T>> for StableDeduped {
    fn ensure(target: &mut Vec<T>) {
        let mut seen = HashSet::new();
        target.retain(|val| seen.insert(val.clone()));
    }
}

pub struct ColorCtor<'a, const Z: u8> {
    pub colors: &'a mut Vec<[f32; 4]>,
}

impl<'a, const Z: u8> ColorCtor<'a, Z> {
    pub fn new(colors: &'a mut Vec<[f32; 4]>) -> Self {
        Self { colors }
    }
}

impl<'a, const Z: u8> FillVertexConstructor<[f32; 3]> for ColorCtor<'a, Z> {
    #[rustfmt::skip]
    fn new_vertex(&mut self, mut vertex: FillVertex) -> [f32; 3] {
        self.colors.push(vertex.interpolated_attributes().try_into().unwrap());
        vertex.position().to_array().pipe(|[x, y]| [x, y, Z as f32])
    }
}

/// This is a hack to make windows function as panels.
/// Tile based UI is difficult to express with ECS functions so this solves that.
/// - Start with the maximal available realestate
/// - Split and subtract the area needed by the current widget
/// - Consume and pipe realestate through systems or allocate to resources used by each system
#[derive(Debug, PartialEq, Eq, Resource)]
pub struct Realestate<T = ()> {
    pub x0: P32,
    pub y0: P32,
    pub x1: P32,
    pub y1: P32,
    _phantom: PhantomData<T>,
}

impl<T> Clone for Realestate<T> {
    fn clone(&self) -> Self {
        Realestate {
            x0: self.x0,
            x1: self.x1,
            y0: self.y0,
            y1: self.y1,
            _phantom: PhantomData,
        }
    }
}

impl<T> Copy for Realestate<T> {}

#[rustfmt::skip]
impl<T> Default for Realestate<T> {
    fn default() -> Self {
        Self { x0: p32(0.), x1: p32(0.), y0: p32(0.), y1: p32(0.), _phantom: PhantomData }
    }
}

#[rustfmt::skip]
impl<T> From<Realestate<T>> for egui::Rect {
    fn from(Realestate { x0, y0, x1, y1, .. }: Realestate<T>) -> Self {
        Self {
            min: egui::Pos2 { x: x0.raw(), y: y0.raw() },
            max: egui::Pos2 { x: x1.raw(), y: y1.raw() }
        }
    }
}

#[rustfmt::skip]
impl<T> Realestate<T> {
    pub fn new((x0, y0): (P32, P32), (x1, y1): (P32, P32)) -> Self {
        assert!(x0 <= x1);
        assert!(y0 <= y1);
        Self { x0, y0, x1, y1, _phantom: PhantomData }
    }

    pub fn into<U>(self) -> Realestate<U> {
        Realestate {
            x0: self.x0,
            x1: self.x1,
            y0: self.y0,
            y1: self.y1,
            _phantom: PhantomData,
        }
    }

    pub fn height(self) -> P32 {
        self.y1 - self.y0
    }

    pub fn width(self) -> P32 {
        self.x1 - self.x0
    }

    pub fn pos(self) -> egui::Pos2 {
        egui::pos2(self.x0.raw(), self.y0.raw())
    }

    pub fn size(self) -> egui::Vec2 {
        egui::vec2(self.width().raw(), self.height().raw())
    }

    pub fn vertical_split<const N: usize>(self, proportions: [P32; N]) -> [Self; N] {
        let (denom, available_x, mut scan_x) = (
            proportions.iter().sum::<P32>(),
            self.x1 - self.x0,
            self.x0
        );

        proportions.map(|p| Self {
            x0: scan_x,
            x1: (scan_x + available_x * (p / denom))
                .raw()
                .clamp(self.x0.raw(), self.x1.raw())
                .pipe(p32)
                .tap(|new_max| scan_x = *new_max),
            ..self
        })
    }

    pub fn horizontal_split<const N: usize>(self, proportions: [P32; N]) -> [Self; N] {
        let (denom, available_y, mut scan_y) = (
            proportions.iter().sum::<P32>(),
            self.y1 - self.y0,
            self.y0
        );

        proportions.map(|p| Self {
            y0: scan_y,
            y1: (scan_y + available_y * (p / denom))
                .raw()
                .clamp(self.y0.raw(), self.y1.raw())
                .pipe(p32)
                .tap(|new_max| scan_y = *new_max),
            ..self
        })
    }
}

// Bug: [https://github.com/emilk/egui/issues/498]
pub fn fixed_layout_bug_workaround(ui: &mut egui::Ui) {
    ui.set_width(ui.available_width());
    ui.set_height(ui.available_height());
}
