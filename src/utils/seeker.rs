pub trait Seeker<Output> {
    fn seek(&mut self, val: f32) -> Output;
    fn jump(&mut self, val: f32) -> Output;
}

pub trait Seekable<'a> {
    type Output;
    type SeekerType: Seeker<Self::Output>;
    fn seeker(&'a self) -> Self::SeekerType;
}





pub struct SimpleAnchor<ValType>
{
    pub offset: f32,
    pub val: ValType
}

impl<ValType> From<(f32, ValType)> for SimpleAnchor<ValType>
{
    fn from(tup: (f32, ValType)) -> SimpleAnchor<ValType> {
        SimpleAnchor::<ValType> {
            offset: tup.0,
            val: tup.1
        }
    }
}

pub struct SimpleSeeker<'a, ValType>
{
    index: usize,
    vec: &'a Vec<SimpleAnchor<ValType>>,
}

trait Interp<Meta, Output> {
    fn interp(&self, meta: Meta) -> Output;
}


impl <'a, ValType> SimpleSeeker<'a, ValType>
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

    pub fn val(&self) -> ValType {    
        self.vec[if self.over_run() { self.vec.len() - 1 } else {self.index}].val
    }

}

impl<'a, ValType> Seeker<ValType> for SimpleSeeker<'a, ValType>
{
    fn seek(&mut self, offset: f32) -> ValType {
        let old = self.index;
        while self.index < self.vec.len() {
            if offset < self.vec[self.index].offset {
                break;
            }
            self.index += 1;
        }
        self.val()
    }

    fn jump(&mut self, offset: f32) -> ValType {
        self.index = match self
            .vec
            .binary_search_by(|elem| elem.offset.partial_cmp(&offset).unwrap())
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
        self.val()
    }
}

impl<'a, ValType> Seekable<'a> for Vec<SimpleAnchor<ValType>>
where
    ValType: 'a,
{
    type Output = ValType;
    type SeekerType = SimpleSeeker<'a, ValType>;
    fn seeker(&'a self) -> Self::SeekerType {
        Self::SeekerType {
            index: 0,
            vec: &self,
        }
    }
}
