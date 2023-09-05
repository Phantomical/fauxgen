use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::Stream;

use crate::GeneratorState;

pub trait AsyncGenerator<A = ()> {
    type Yield;
    type Return;

    fn poll_resume(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        arg: Option<A>,
    ) -> Poll<GeneratorState<Self::Yield, Self::Return>>;

    fn resume(self: Pin<&mut Self>, arg: A) -> Resume<A, Self> {
        Resume {
            arg: Some(arg),
            gen: self,
        }
    }
}

pub trait AsyncGeneratorExt<A = ()>: AsyncGenerator<A> {
    fn stream(self) -> GenStream<Self>
    where
        Self: Sized,
    {
        GenStream(self)
    }
}

impl<A, G> AsyncGeneratorExt<A> for G where G: AsyncGenerator<A> {}

pub struct GenStream<G>(G);

impl<G> Stream for GenStream<G>
where
    G: AsyncGenerator<(), Return = ()>,
{
    type Item = G::Yield;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let gen = unsafe { self.map_unchecked_mut(|s| &mut s.0) };
        gen.poll_resume(cx, Some(())).map(|state| match state {
            GeneratorState::Yield(value) => Some(value),
            GeneratorState::Return(()) => None,
        })
    }
}

pub struct Resume<'g, A, G: ?Sized> {
    gen: Pin<&'g mut G>,
    arg: Option<A>,
}

impl<'g, A, G> Future for Resume<'g, A, G>
where
    G: AsyncGenerator<A> + ?Sized,
{
    type Output = GeneratorState<G::Yield, G::Return>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        this.gen.as_mut().poll_resume(cx, this.arg.take())
    }
}

impl<'g, A, G: ?Sized> Unpin for Resume<'g, A, G> {}
