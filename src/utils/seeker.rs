pub trait Seeker<Output> {
    fn advance(&mut self, offset: f32) -> Output;
    fn jump(&mut self, offset: f32) -> Output;
}

pub trait Seekable<Output> {
    type Seeker: Seeker<Output>;
    fn start(&self, offset: f32) -> Self::Seeker;
    fn jump(&self, offest: f32) -> Self::Seeker;
}
