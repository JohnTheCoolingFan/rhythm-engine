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
#[derive(Clone, Copy, Default)]
pub struct Epoch<Value> {
    pub offset: f32,
    pub val: Value,
}

impl<Value> Quantify for Epoch<Value>
where
    Value: Copy,
{
    type Quantifier = f32;
    fn quantify(&self) -> Self::Quantifier {
        self.offset
    }
}

impl<Value> From<(f32, Value)> for Epoch<Value>
where
    Value: Copy,
{
    fn from(tup: (f32, Value)) -> Epoch<Value> {
        Epoch::<Value> {
            offset: tup.0,
            val: tup.1,
        }
    }
}

#[duplicate(
    VecT                D;
    [Vec<Epoch<T>>]     [];
    [TVec<Epoch<T>>]   [Default]
)]
impl<'a, T> SeekerTypes for Seeker<&'a VecT, usize> 
where
    T: Copy,
    Epoch<T>: D
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

pub type Output<'a, T> = <T as SeekerTypes>::Output;
pub type Quantifier<'a, T> = <<T as SeekerTypes>::Source as Quantify>::Quantifier;

//  T<U> is not possible so I have to do this
duplicate_inline! {
    [VecT       D;
    [Vec<T>]    [];
    [TVec<T>]  [Default]]
    
    impl<'a, T> Seeker<&'a VecT, usize>
    where
        T: Quantify + D
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

    impl<'a, T> Seek for Seeker<&'a VecT, usize>
    where
        T: Quantify + D,
        Self: Exhibit<Source = T>
    {
        fn seek(&mut self, offset: Quantifier<'a, Self>) -> Output<'a, Self>
        {
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

    impl <'a, T> Seekable<'a> for VecT
    where
        T: Quantify + 'a + D,
        Seeker<&'a VecT, usize>: Exhibit<Source = T>,
    {
        type Seeker = Seeker<&'a VecT, usize>;
        
        fn seeker(&'a self) -> Self::Seeker {
            Self::Seeker {
                meta: 0,
                data: self
            }
        }
    }


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
}
//
//
//
//
//
#[derive(Clone, Copy)]
pub enum Transition {
    Instant,
    Weighted(f32)
}

impl Transition {
    pub fn cycle(&mut self) {
        *self = match self {
            Self::Instant => Self::Weighted(0.),
            Self::Weighted(_) => Self::Instant
        }
    }
}

