use std::pin::Pin;

use crate::{Generator, GeneratorState};

pub trait GeneratorExt<A = ()>: Generator<A> {
    fn iter(self) -> GenIter<Self>
    where
        Self: Unpin + Sized,
    {
        GenIter(self)
    }
}

impl<A, G> GeneratorExt<A> for G where G: Generator<A> {}
pub struct GenIter<G>(G);

impl<G> Iterator for GenIter<G>
where
    G: Generator<(), Return = ()> + Unpin,
{
    type Item = G::Yield;

    fn next(&mut self) -> Option<Self::Item> {
        match Pin::new(&mut self.0).resume(()) {
            GeneratorState::Return(()) => None,
            GeneratorState::Yield(value) => Some(value),
        }
    }
}
