pub struct FromEnd(pub usize);

use duplicate::duplicate;
#[duplicate(itterable; [Vec<T>]; [[T]])]
impl<T> std::ops::Index<FromEnd> for itterable {
    type Output = T;

    fn index(&self, FromEnd(n): FromEnd) -> &T {
        &self[self.len().checked_sub(1 + n).expect("out of range from end")]
    }
}
