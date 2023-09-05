use std::pin::Pin;

use crate::{generator, Generator, GeneratorState};

used_in_docs!(generator);

/// Wrapper around a generator that implements [`Iterator`].
///
/// The generators created by the [`generator`] macro implement [`Iterator`]
/// once they are pinned. For other implementations of [`Generator`], though,
/// you can use `GeneratorIter` to convert them into an interator.
pub struct GeneratorIter<G>(G);

impl<G> GeneratorIter<G> {
    pub fn new(gen: G) -> Self {
        Self(gen)
    }

    pub fn into_inner(self) -> G {
        self.0
    }
}

impl<G> Iterator for GeneratorIter<G>
where
    G: Generator<(), Return = ()> + Unpin,
{
    type Item = G::Yield;

    fn next(&mut self) -> Option<Self::Item> {
        match Pin::new(&mut self.0).resume(()) {
            GeneratorState::Complete(()) => None,
            GeneratorState::Yielded(value) => Some(value),
        }
    }
}
