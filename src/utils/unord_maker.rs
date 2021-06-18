//needs better names
pub trait MakerPart {
    type Complete;
    
    fn add(self) -> Self;
    fn try_make(self) -> Option<Self::Complete>;
}

//pass in closure it calls to get part info???
pub trait UnOrdMaker {
    type Part: MakerPart<Complete = Self>;

    fn new() -> Self;
    fn extend(&mut self, part: Self::Part);
}
