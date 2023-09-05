use std::{pin::Pin, task::{Context, Poll}};

use futures_core::Stream;

use crate::{AsyncGenerator, GeneratorState};

pub struct GeneratorStream<G>(G);

impl<G> GeneratorStream<G> {
    pub fn new(gen: G) -> Self {
        Self(gen)
    }

    pub fn into_inner(self) -> G {
        self.0
    }
}

impl<G> Stream for GeneratorStream<G>
where
    G: AsyncGenerator<(), Return = ()>,
{
    type Item = G::Yield;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let gen = unsafe { self.map_unchecked_mut(|s| &mut s.0) };
        gen.poll_resume(cx, Some(())).map(|state| match state {
            GeneratorState::Yielded(value) => Some(value),
            GeneratorState::Complete(()) => None,
        })
    }
}
