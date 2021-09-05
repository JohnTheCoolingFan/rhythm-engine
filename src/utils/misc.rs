use duplicate::duplicate_inline;
use std::fmt::Debug;
use tinyvec::TinyVec;
use std::default::Default;
use glam::Vec2;

pub const SHORT_ARR_SIZE: usize = 3;
pub type TVec<T> = TinyVec<[T; SHORT_ARR_SIZE]>;

pub struct FromEnd(pub usize);

//  T<U> is not possible so I have to do this
duplicate_inline! {
    [VecT       D;
    [Vec<T>]    [];
    [TVec<T>]  [Default]]

    impl<T> std::ops::Index<FromEnd> for VecT
    where
        T: D
    {
        type Output = T;

        fn index(&self, FromEnd(n): FromEnd) -> &T {
            &self[self
                .len()
                .checked_sub(1 + n)
                .expect(format!("From End out of range. Item len: {}", self.len()).as_str())]
        }
    }

    impl<T> std::ops::IndexMut<FromEnd> for VecT
    where
        T: D
    {
        fn index_mut(&mut self, FromEnd(n): FromEnd) -> &mut T {
            let len = self.len();
            &mut self[len.checked_sub(1 + n).expect("out of range from end")]
        }
    }
}

pub trait ShortHandDebug: Debug + Copy {
    fn debug(self, ident: &str) -> Self {
        println!("{}: {:?}", ident, self);
        self
    }
}

impl<T: Debug + Copy> ShortHandDebug for T {}
