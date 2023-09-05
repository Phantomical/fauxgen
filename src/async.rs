use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

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
