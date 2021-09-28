use crate::utils::misc::*;
use duplicate::*;
use std::default::Default;
//for values to seek over
pub trait Quantify {
    type Quantifier: PartialOrd;

    fn quantify(&self) -> Self::Quantifier;
}

//for seeker
pub trait SeekerTypes {
    type Source: Quantify; //in case of meta seekers this is the leader
    type Output;
}

pub trait Seek: SeekerTypes {
    fn seek(&mut self, offset: <Self::Source as Quantify>::Quantifier) -> Self::Output;
    fn jump(&mut self, offset: <Self::Source as Quantify>::Quantifier) -> Self::Output;
}

pub trait Exhibit: SeekerTypes {
    fn exhibit(& self, t: <Self::Source as Quantify>::Quantifier) -> Self::Output;
}

//for collection of seekable values
//lifetime is needed because some impls will have uncoinstrained lifetime otherwise
pub trait Seekable<'a> {
    type Seeker: Seek;
    
    fn seeker(&'a self) -> Self::Seeker;
}

pub trait SeekExtensions{
    type Item: Quantify;
    fn quantified_insert(&mut self, item: Self::Item) -> usize;
}
//
//
//
//
//
#[derive(Clone, Copy, Default, Debug)]
pub struct Epoch<Value> {
    pub offset: f32,
    pub val: Value,
}

pub struct Seeker<Data, Meta>
{
    pub data: Data, //unchanging
    pub meta: Meta, //changing
}

pub type Output<'a, T> = <T as SeekerTypes>::Output;
pub type Quantifier<'a, T> = <<T as SeekerTypes>::Source as Quantify>::Quantifier;
//
//
//
//
//
impl<Value> Quantify for Epoch<Value> {
    type Quantifier = f32;
    fn quantify(&self) -> Self::Quantifier {
        self.offset
    }
}

impl<Value> From<(f32, Value)> for Epoch<Value> {
    fn from(tup: (f32, Value)) -> Epoch<Value> {
        Epoch::<Value> {
            offset: tup.0,
            val: tup.1,
        }
    }
}

impl<'a, T> SeekerTypes for Seeker<&'a [Epoch<T>], usize> 
where
    T: Copy,
{
    type Source = Epoch<T>;
    type Output = T;
}
//
//
//
//
//
impl<'a, T> Seekable<'a> for [T]
where
    T: Quantify + 'a,
    Seeker<&'a [T], usize>: Seek<Source = T>
{
    type Seeker = Seeker<&'a [T], usize>;
    
    fn seeker(&'a self) -> Self::Seeker {
        Self::Seeker {
            meta: 0,
            data: self
        }
    }
}

impl<'a, T> Seeker<&'a [T], usize>
where
    T: Quantify
{
    pub fn current(&self) -> Result<&T, &T> {
        if self.meta < self.data.len() {
            Ok(&self.data[self.meta])
        }
        else {
            Err(&self.data[FromEnd(0)])
        }
    }

    pub fn previous(&self) -> Option<&T> {
        if 1 < self.data.len() && 0 < self.meta {
            Some(&self.data[self.meta - 1])
        }
        else {
            None
        }
    }

    pub fn next(&self) -> Option<&T> {
        if 1 < self.data.len() && self.meta - 1 < self.data.len() {
            Some(&self.data[self.meta + 1])
        }
        else {
            None
        }
    }
}

impl<'a, T> Seek for Seeker<&'a [T], usize>
where
    T: Quantify,
    Self: Exhibit<Source = T>
{
    fn seek(&mut self, offset: Quantifier<'a, Self>) -> Output<'a, Self> {
        while self.meta < self.data.len() {
            if offset < self.data[self.meta].quantify() {
                break;
            }
            self.meta += 1;
        }
        self.exhibit(offset)
    }

    fn jump(&mut self, offset: Quantifier<'a, Self>) -> Output<'a, Self> {
        self.meta = match self
            .data
            .binary_search_by(|elem| elem.quantify().partial_cmp(&offset).unwrap())
        {
            Ok(index) => index,
            Err(index) => index 
        };
        self.exhibit(offset)
    }
}
//
//
//
//
//
#[duplicate(
    VecT        D;
    [Vec<T>]    [];
    [TVec<T>]   [Default]
)]
impl<T> SeekExtensions for VecT
where
    T: Quantify + Copy + D,
{
    type Item = T;
    fn quantified_insert(&mut self, item: T) -> usize {
        let index = match self
            .as_slice()
            .binary_search_by(|a| a.quantify().partial_cmp(&item.quantify()).unwrap()) {
                Ok(index) | Err(index) => index,
        };
        self.insert(index, item);
        index
    }
}
