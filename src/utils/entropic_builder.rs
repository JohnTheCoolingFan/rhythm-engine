pub trait EntropicPart {
    type Complete;
    fn add(self) -> Self;
    fn build(self) -> Self::Complete;
}

pub trait EntropicBuilder {
    type Part: EntropicPart<Complete = Self>;
    fn entrpc_builder() -> Self::Part;
    fn entrpc_extend(self) -> Self::Part;
}
