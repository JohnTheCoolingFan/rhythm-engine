use crate::utils::misc::*;
use std::ops::{Index, IndexMut};

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
pub trait Exhibit: SeekerTypes {
    fn exhibit(&self, t: <Self::Source as Quantify>::Quantifier) -> Self::Output;
}

pub trait Seek: SeekerTypes {
    fn seek(&mut self, offset: <Self::Source as Quantify>::Quantifier) -> Self::Output;
    fn jump(&mut self, offset: <Self::Source as Quantify>::Quantifier) -> Self::Output;
}

//for collection of seekable values
pub trait Seekable<'a> {
    type Seeker: Seek;
    
    fn seeker(&'a self) -> Self::Seeker;
}

pub trait SeekExtensions
{
    type Item: Quantify;

    fn quantified_insert(&mut self, item: Self::Item) -> usize;
}
//
//
//
//
//
#[derive(Clone, Copy)]
pub struct Epoch<Value> {
    pub time: f32,
    pub val: Value,
}

impl<Value> Quantify for Epoch<Value>
where
    Value: Copy,
{
    type Quantifier = f32;
    fn quantify(&self) -> Self::Quantifier {
        self.time
    }
}

impl<Value> From<(f32, Value)> for Epoch<Value>
where
    Value: Copy,
{
    fn from(tup: (f32, Value)) -> Epoch<Value> {
        Epoch::<Value> {
            time: tup.0,
            val: tup.1,
        }
    }
}

impl<T> SeekerTypes for Seeker<Epoch<T>, usize> 
where
    T: Copy
{
    type Source = Epoch<T>;
    type Output = T;
}

//
//
//
//
//
pub struct Seeker<Data, Meta>
where
{
    pub data: Data, //unchanging
    pub meta: Meta, //changign
}

pub type DataItem<'a, T> = <<T as Seekable<'a>>::Seeker as SeekerTypes>::Output;

impl<'a, Data> Seeker<&'a Data, usize>
where
    Data: Seekable<'a>+ Index<usize, Output = DataItem<'a, Data>> + 'a
{
    pub fn current(&self) -> Result<&DataItem<'a, Data>, &DataItem<'a, Data>> {
        if self.meta < self.data.len() {
            Ok(&self.data[self.meta])
        }
        else {
            Err(&self.data[FromEnd(0)])
        }
    }

    pub fn previous(&self) -> Option<&DataItem<'a, Data>> {
        if 1 < self.data.len() && 0 < self.meta {
            Some(&self.data[self.meta - 1])
        }
        else {
            None
        }
    }

    pub fn next(&self) -> Option<&DataItem<'a, Data>> {
        if 1 < self.data.len() && self.meta - 1 < self.data.len() {
            Some(&self.data[self.meta + 1])
        }
        else {
            None
        }
    }
}

impl<'a, Item> Seek for BPSeeker<'a, Item>
where
    Item: Quantify,
    Self: Exhibit<Source = Item>,
{
    fn seek(&mut self, offset: <Self::Source as Quantify>::Quantifier) -> Self::Output {
        while self.meta < self.data.len() {
            if offset < self.data[self.meta].quantify() {
                break;
            }
            self.meta += 1;
        }
        self.exhibit(offset)
    }

    fn jump(&mut self, offset: <Self::Source as Quantify>::Quantifier) -> Self::Output {
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

impl <'a, Item> Seekable<'a> for Vec<Item>
where
    Item: Quantify + 'a,
    BPSeeker<'a, Item>: Exhibit<Source = Item>,
{
    type Seeker = Seeker<&'a Vec<Item>, usize>;
    
    fn seeker(&'a self) -> Self::Seeker {
        Self::Seeker {
            meta: 0,
            data: self
        }
    }
}

impl<T> SeekExtensions for Vec<T>
where
    T: Quantify + Copy,
{
    type Item = T;
    fn quantified_insert(&mut self, item: T) -> usize {
        let index = match self.binary_search_by(|a| a.quantify().partial_cmp(&item.quantify()).unwrap()) {
            Ok(index) | Err(index) => index,
        };
        self.insert(index, item);
        index
    }
}
