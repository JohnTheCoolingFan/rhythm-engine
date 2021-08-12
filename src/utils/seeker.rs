use crate::utils::misc::*;

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

impl<'a, T> SeekerTypes for BPSeeker<'a, Epoch<T>> 
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

//BP prefix means "boiler plate"
pub type BPSeeker<'a, Item> = Seeker<&'a Vec<Item>, usize>;

impl<'a, Item> BPSeeker<'a, Item>
where
{
    pub fn index(&self) -> usize {
        self.meta
    }

    pub fn vec(&self) -> &Vec<Item> {
        &self.data
    }
 
    pub fn over_run(&self) -> bool {
        self.data.len() <= self.meta
    }
    
    pub fn under_run(&self) -> bool {
        self.meta == 0
    }

    pub fn current(&self) -> &Item {
        if self.meta < self.data.len() {
            &self.data[self.meta]
        }
        else {
            &self.data[FromEnd(0)]
        }
    }

    pub fn previous(&self) -> &Item {
        if 0 == self.meta {
            &self.data[0]
        }
        else if self.data.len() <= self.meta {
            &self.data[FromEnd(0)]
        }
        else {
            &self.data[self.meta - 1]
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
            Err(index) => {
                if self.data.len() < index {
                    index
                } else {
                    self.data.len()
                }
            }
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
            data: &self
        }
    }
}

impl<T> SeekExtensions for Vec<T>
where
    T: Quantify,
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
