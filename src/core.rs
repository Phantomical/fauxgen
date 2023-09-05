use std::pin::Pin;


pub trait Generator<R = ()> {
    type Yield;
    type Return;

    fn resume(self: Pin<&mut Self>, arg: R) -> GeneratorState<Self::Yield, Self::Return>;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GeneratorState<Y, R> {
    Yield(Y),
    Return(R),
}
