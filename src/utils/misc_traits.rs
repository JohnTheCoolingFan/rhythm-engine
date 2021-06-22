pub enum ButtonState {
    Press,
    Held,
    Release
}
pub trait MouseHandle<T> {
    fn mouse_event(&self, info: T);
}

pub trait Seeker<Output> {
    fn seek(&mut self, val: f32) -> Output;
    fn jump(&mut self, val: f32) -> Output;
}

pub trait Seekable<'a, Output> {
    type Seeker: Seeker<Output>;
    fn seeker(&'a self) -> Self::Seeker;
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
            .expect("out of range from end")]
    }
}

#[duplicate(itterable; [Vec<T>]; [[T]])]
impl<T> std::ops::IndexMut<FromEnd> for itterable {
    fn index_mut(&mut self, FromEnd(n): FromEnd) -> &mut T {
        let len = self.len();
        &mut self[len.checked_sub(1 + n).expect("out of range from end")]
    }
}
