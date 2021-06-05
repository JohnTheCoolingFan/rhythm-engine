pub trait Seeker<Output> {
    fn seek(&mut self, val: f32) -> Output;
    fn jump(&mut self, val: f32) -> Output;
}

pub trait Seekable<'a, Output> {
    type Seeker: Seeker<Output>;
    fn seeker(&'a self) -> Self::Seeker;
}
