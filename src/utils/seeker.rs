pub trait Seeker<Output> {
    fn seek(&mut self, offset: f32) -> Output;
    fn jump(&mut self, offset: f32) -> Output;
}

pub trait Seekable<Output> {
    type Seeker: Seeker<Output>;
    fn seeker(&self) -> Self::Seeker;
}
