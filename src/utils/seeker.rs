pub trait Seeker<Output> {
    fn seek(&mut self, val: f32) -> Output;
    fn jump(&mut self, val: f32) -> Output;
}

pub trait Seekable<'a> {
    type Output;
    type SeekerType: Seeker<Self::Output>;
    fn seeker(&'a self) -> Self::SeekerType;
}

#[derive(Clone, Copy)]
pub struct SeekableQuantum<T>
where
    T: Copy
{
    val: T,
}

impl <T> From<T> for SeekableQuantum<T>
where
    T: Copy
{
    fn from(t: T) -> SeekableQuantum<T> {
        SeekableQuantum::<T> {
            val: t
        }
    }
}

impl<T> Seeker<T> for SeekableQuantum<T>
where
    T: Copy
{
    fn seek(&mut self, _offset: f32) -> T {
        self.val
    }
    fn jump(&mut self, _offset: f32) -> T {
        self.val
    }
}

impl<'a, T> Seekable<'a> for SeekableQuantum<T> 
where
    T: Copy
{
    type Output = T;
    type SeekerType = Self;
    fn seeker(&'a self) -> Self::SeekerType {
        *self
    }
}

pub struct BlanketSeeker<'a, T>
where
    T: Seekable<'a>,
{
    index: usize,
    seeker: T::SeekerType,
    vec: &'a Vec<(f32, T)>,
}

impl <'a, T> BlanketSeeker<'a, T>
where
    T: Seekable<'a, SeekerType = T> + Seeker<T::Output>,
{
    pub fn dead_get(&mut self) -> T::Output {
        self.seeker.seek(f32::NAN)
    }
}

impl<'a, T> Seeker<T::Output> for BlanketSeeker<'a, T>
where
    T: Seekable<'a>,
{
    fn seek(&mut self, offset: f32) -> T::Output {
        let old = self.index;
        while self.index < self.vec.len() {
            if offset < self.vec[self.index].0 {
                break;
            }
            self.index += 1;
        }
        if old != self.index {
            self.seeker = self.vec[self.index].1.seeker();
        }
        self.seeker.seek(offset - self.vec[self.index].0)
    }

    fn jump(&mut self, offset: f32) -> T::Output {
        self.index = match self
            .vec
            .binary_search_by(|elem| elem.0.partial_cmp(&offset).unwrap())
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

        self.seeker = self.vec[self.index].1.seeker();
        self.seeker.jump(offset - self.vec[self.index].0)
    }
}

impl<'a, T> Seekable<'a> for Vec<(f32, T)>
where
    T: Seekable<'a> + 'a,
{
    type Output = T::Output;
    type SeekerType = BlanketSeeker<'a, T>;
    fn seeker(&'a self) -> Self::SeekerType {
        Self::SeekerType {
            index: 0,
            seeker: self[0].1.seeker(),
            vec: &self,
        }
    }
}

pub trait GetQuant<T> {
    fn quantum(&self) -> &T;
}

impl <T> GetQuant<T> for (f32, SeekableQuantum<T>) 
where
    T: Copy
{
    fn quantum(&self) -> &T {
        &self.1.val
    }
}
