pub struct FromEnd(pub usize);
use duplicate::duplicate;
use std::fmt::Debug;

#[duplicate(itterable; [Vec<T>]; [[T]])]
impl<T> std::ops::Index<FromEnd> for itterable {
    type Output = T;

    fn index(&self, FromEnd(n): FromEnd) -> &T {
        &self[self
            .len()
            .checked_sub(1 + n)
            .expect(format!("From End out of range. Item len: {}", self.len()).as_str())]
    }
}

#[duplicate(itterable; [Vec<T>]; [[T]])]
impl<T> std::ops::IndexMut<FromEnd> for itterable {
    fn index_mut(&mut self, FromEnd(n): FromEnd) -> &mut T {
        let len = self.len();
        &mut self[len.checked_sub(1 + n).expect("out of range from end")]
    }
}

pub trait ShortHandDebug: Debug + Copy {
    fn debug(self, ident: &str) -> Self {
        println!("{}: {:?}", ident, self);
        self
    }
}

impl<T: Debug + Copy> ShortHandDebug for T {}

#[derive(Clone, Copy)]
pub enum Interpret<T> 
where
    T: Copy
{
    Individual(T),
    Respective(T)
}

impl<T> Interpret<T> 
where
    T: Copy
{
    pub fn get(&self) -> &T {
        match self {
            Self::Respective(val) | Self::Individual(val) => val
        }
    }

    pub fn get_mut(&mut self) -> &mut T {
        match self {
            Self::Respective(val) | Self::Individual(val) => val
        }
    }

    pub fn cycle(&mut self) {
        *self = match self {
            Self::Respective(val) => Self::Individual(*val),
            Self::Individual(val) => Self::Respective(*val)
        }
    }
}

