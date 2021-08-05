pub trait Quantify {
    type Quantifier: PartialOrd;
    
    fn quantify(&self) -> Self::Quantifier;
}

pub trait Exhibit {
    type Item: Quantify;
    type Output;
    
    fn exhibit(
        &self, 
        t: <Self::Item as Quantify>::Quantifier
    ) -> Self::Output;
}

trait Seek {
    type Item: Quantify + Exhibit;
    
    fn index(&self) -> usize;
    fn over_run(&self) -> bool;
    fn under_run(&self) -> bool;
    fn val(&self) -> &Self::Item;
    fn seek(
        &mut self,
        offset: <Self::Item as Quantify>::Quantifier
    ) -> <Self::Item as Exhibit>::Output;
    fn jump(
        &mut self,
        offset: <Self::Item as Quantify>::Quantifier
    ) -> <Self::Item as Exhibit>::Output;
}

pub trait SeekExtensions {
    type Seekable: Quantify;

    fn se_insert(&mut self, item: Self::Seekable);
    fn se_remove(&mut self, index: usize) -> Result<Self::Seekable, usize>;
    fn seeker(&self) -> Seeker<Self::Seekable>;
}
//
//
//
//
//
pub struct Seeker<'a, Item>
where
    Item: Quantify + Exhibit
{
    index: usize,
    vec: &'a Vec<Item>,
}

impl<'a, Item> Seek for Seeker<'a, Item>
where
    Item: Quantify + Exhibit
{
    type Item = Item;

    fn index(&self) -> usize {
        self.index
    }

    fn over_run(&self) -> bool {
        self.vec.len() <= self.index
    }

    fn under_run(&self) -> bool {
        self.index == 0
    }

    fn val(&self) -> &Item {
        &self.vec[if self.over_run() {
            self.vec.len() - 1
        } else {
            self.index
        }]
    }

    fn index(&self) -> usize {
        self.index
    }

    fn seek(&mut self, offset: Item::Quantifier) -> Item::Output {
        while self.index < self.vec.len() {
            if offset < self.vec[self.index].quantify() {
                break;
            }
            self.index += 1;
        }
        self.exhibit(offset)
    }

    fn jump(&mut self, offset: Item::Quantifier) -> Item::Output {
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
pub struct Epoch<Value>
{
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

impl<Value> Exhibit for Epoch<Value>
where
    Value: Copy
{
    type Item = Self;
    type Output = Value;

    fn exhibit(&self, _: f32) -> Self::Output {
        self.val
    }
}

impl <Value> Quantify for Epoch<Value>
where
    Value: Seek
{
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
impl<T: Seek> SeekExtensions for Vec<T> {
    type Seekable = T;

    fn se_insert(&mut self, item: Self::Seekable) {
        self.insert(
            match self.binary_search_by(|a| a.quantify().partial_cmp(&item.quantify()).unwrap()) {
                Ok(index) | Err(index) => index,
            },
            item,
        );
    }

    fn se_remove(&mut self, index: usize) -> Result<Self::Seekable, usize> {
        if index < self.len() {
            Ok(self.remove(index))
        } else {
            Err(index)
        }
    }

    fn seeker(&self) -> Seeker<Self::Seekable> {
        Seeker::<Self::Seekable> {
            index: 0,
            vec: &self,
        }
    }
}
