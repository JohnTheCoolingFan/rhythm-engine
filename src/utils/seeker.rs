pub trait Seekable: Sized {
    type Output;
    type Quantifier: PartialOrd;
    fn quantify(&self) -> Self::Quantifier;
    fn exhibit(&self, t: Self::Quantifier, seeker: &Seeker<Self>) -> Self::Output;
}

pub trait SeekingExtensions {
    type Seekable: Seekable;

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
    Item: Seekable,
{
    index: usize,
    vec: &'a Vec<Item>,
}

impl<'a, Item> Seeker<'a, Item>
where
    Item: Seekable,
{
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn over_run(&self) -> bool {
        self.vec.len() <= self.index
    }

    pub fn under_run(&self) -> bool {
        self.index == 0
    }

    pub fn val(&self) -> &Item {
        &self.vec[if self.over_run() {
            self.vec.len() - 1
        } else {
            self.index
        }]
    }

    pub fn vec(&self) -> &Vec<Item> {
        &self.vec
    }

    pub fn index(&self) -> usize {
        self.index
    }

    fn seek(&mut self, offset: Item::Quantifier) -> Item::Output {
        while self.index < self.vec.len() {
            if offset < self.vec[self.index].quantify() {
                break;
            }
            self.index += 1;
        }
        self.val().exhibit(offset, &self)
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
        self.val().exhibit(offset, &self)
    }
}
//
//
//
//
//
pub struct Epoch<Value>
where
    Value: Copy,
{
    pub time: f32,
    pub val: Value,
}

impl<Value> Seekable for Epoch<Value>
where
    Value: Copy,
{
    type Output = Value;
    type Quantifier = f32;
    fn quantify(&self) -> Self::Quantifier {
        self.time
    }
    fn exhibit(&self, _t: Self::Quantifier, _s: &Seeker<Self>) -> Self::Output {
        self.val
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
impl<T: Seekable> SeekingExtensions for Vec<T> {
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
