//for values to seek over
pub trait Quantify {
    type Quantifier: PartialOrd;

    fn quantify(&self) -> Self::Quantifier;
}

//for seeker of values
pub trait Exhibit {
    type Source: Quantify;
    type Output;

    fn exhibit(&self, t: <Self::Source as Quantify>::Quantifier) -> Self::Output;
}

//for seeker of values
pub trait Seek: Exhibit {
    fn index(&self) -> usize;
    fn over_run(&self) -> bool;
    fn under_run(&self) -> bool;
    fn get(&self) -> &Self::Source;
    fn seek(&mut self, offset: <Self::Source as Quantify>::Quantifier) -> Self::Output;
    fn jump(&mut self, offset: <Self::Source as Quantify>::Quantifier) -> Self::Output;
}

//for collection of seekable values
pub trait SeekExtensions<'a> {
    type Seeker: Seek;

    fn se_insert(&mut self, item: <Self::Seeker as Exhibit>::Source);
    fn se_remove(&mut self, index: usize) -> Result<<Self::Seeker as Exhibit>::Source, usize>;
    fn seeker(&'a self) -> Self::Seeker;
}

//for super types that have a primative seekable container within them that alter the yielded values
pub trait MetaSeek {
    type Leader: Seek;
    type Meta: Copy;
    type Output;

    fn leader(&self) -> Self::Leader;
    fn meta(&self) -> Meta;
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

//
//
//
//
//
pub struct Seeker<'a, Item>
where
    Item: Quantify,
{
    index: usize,
    vec: &'a Vec<Item>,
}

impl<'a, Item> Seek for Seeker<'a, Item>
where
    Item: Quantify,
    Self: Exhibit<Source = Item>,
{
    fn index(&self) -> usize {
        self.index
    }

    fn over_run(&self) -> bool {
        self.vec.len() <= self.index
    }

    fn under_run(&self) -> bool {
        self.index == 0
    }

    fn get(&self) -> &Self::Source {
        &self.vec[if self.over_run() {
            self.vec.len() - 1
        } else {
            self.index
        }]
    }

    fn seek(&mut self, offset: <Self::Source as Quantify>::Quantifier) -> Self::Output {
        while self.index < self.vec.len() {
            if offset < self.vec[self.index].quantify() {
                break;
            }
            self.index += 1;
        }
        self.exhibit(offset)
    }

    fn jump(&mut self, offset: <Self::Source as Quantify>::Quantifier) -> Self::Output {
        self.index = match self
            .vec
            .binary_search_by(|elem| elem.quantify().partial_cmp(&offset).unwrap())
        {
            Ok(index) => index,
            Err(index) => {
                if self.vec.len() < index {
                    index
                } else {
                    self.vec.len()
                }
            }
        };
        self.exhibit(offset)
    }
}
//
//
//
//
//
impl<'a, T: 'a + Quantify> SeekExtensions<'a> for Vec<T>
where
    Seeker<'a, T>: Exhibit<Source = T>,
{
    type Seeker = Seeker<'a, T>;

    fn se_insert(&mut self, item: T) {
        self.insert(
            match self.binary_search_by(|a| a.quantify().partial_cmp(&item.quantify()).unwrap()) {
                Ok(index) | Err(index) => index,
            },
            item,
        );
    }

    fn se_remove(&mut self, index: usize) -> Result<T, usize> {
        if index < self.len() {
            Ok(self.remove(index))
        } else {
            Err(index)
        }
    }

    fn seeker(&'a self) -> Self::Seeker {
        Self::Seeker {
            index: 0,
            vec: &self,
        }
    }
}
