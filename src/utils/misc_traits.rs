pub trait Seeker<Output> {
    fn seek(&mut self, val: f32) -> Output;
    fn jump(&mut self, val: f32) -> Output;
}

pub trait Seekable<'a> {
    type Output;
    type SeekerType: Seeker<Self::Output>;
    fn seeker(&'a self) -> Self::SeekerType;
}



pub struct FromEnd(pub usize);
use duplicate::duplicate;

#[duplicate(itterable; [Vec<T>]; [[T]])]
impl<T> std::ops::Index<FromEnd> for itterable {
    type Output = T;

    fn index(&self, FromEnd(n): FromEnd) -> &T {
        &self[self
            .len()
            .checked_sub(1 + n)
            .expect(format!(
                "From End out of range. Item len: {}",
                self.len()
            ).as_str())]
    }
}

#[duplicate(itterable; [Vec<T>]; [[T]])]
impl<T> std::ops::IndexMut<FromEnd> for itterable {
    fn index_mut(&mut self, FromEnd(n): FromEnd) -> &mut T {
        let len = self.len();
        &mut self[len.checked_sub(1 + n).expect("out of range from end")]
    }
}

