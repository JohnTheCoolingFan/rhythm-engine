///this trait is sort of a pseudo generator
///it implements a few functions that calculate offset dependant values
///all the functions cache informaton about their last call such that
///continuing from that last call with a small time delta is low cost
pub trait Seekable<Value> {
    ///implementation should use binary search as this is for jumps
    fn seek(&self, offset: f32) -> Value;
    
    ///implementation should use linear search as this is expected 
    ///to be called at offsets close to the start
    fn start(&self, offset: f32) -> Value;
    
    ///implementation should use the cached informato to advacnce small time deltas
    ///should not be called before start or seek are called  
    ///should not be called for a large time delta
    fn advance(&self, offset: f32) -> Value;
}
