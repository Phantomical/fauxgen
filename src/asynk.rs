use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::Stream;

use crate::common::GeneratorArg;
use crate::waker::GeneratorWaker;
use crate::{GeneratorState, TokenId};

pub trait AsyncGenerator<A = ()> {
    type Yield;
    type Return;

    fn poll_resume(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        arg: Option<A>,
    ) -> Poll<GeneratorState<Self::Yield, Self::Return>>;
}

pub trait AsyncGeneratorExt<A = ()>: AsyncGenerator<A> {
    fn resume(self: Pin<&mut Self>, arg: A) -> Resume<A, Self>
    where
        Self: Sized,
    {
        Resume {
            arg: Some(arg),
            gen: self,
        }
    }

    fn stream(self) -> GenStream<Self>
    where
        Self: Sized,
    {
        GenStream(self)
    }
}

impl<A, G> AsyncGeneratorExt<A> for G where G: AsyncGenerator<A> {}

pub struct GeneratorWrapper<F, Y, A> {
    future: F,
    arg: GeneratorArg<Y, A>,
    id: TokenId,
}

impl<F, Y, A> GeneratorWrapper<F, Y, A> {
    pub fn new(future: F) -> Self {
        Self {
            future,
            arg: GeneratorArg::Empty,
            id: std::ptr::null(),
        }
    }
}

impl<F, Y, A> AsyncGenerator<A> for GeneratorWrapper<F, Y, A>
where
    F: Future,
{
    type Yield = Y;
    type Return = F::Output;

    fn poll_resume(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        param: Option<A>,
    ) -> Poll<GeneratorState<Self::Yield, Self::Return>> {
        // Manual pin projection
        let this = unsafe { self.get_unchecked_mut() };
        let arg = &mut this.arg;
        let future = unsafe { Pin::new_unchecked(&mut this.future) };

        if let Some(param) = param {
            *arg = GeneratorArg::Arg(param);
        }

        // SAFETY: GeneratorWaker's clone impl returns a different waker so none of the
        //         references stored in waker will outlive this function.
        let waker = unsafe { GeneratorWaker::new(Some(cx.waker()), arg, &mut this.id) };

        // SAFETY: waker will not outlive this function.
        let waker = unsafe { waker.to_waker() };

        let mut context = Context::from_waker(&waker);
        match future.poll(&mut context) {
            Poll::Pending => match arg.take_yield() {
                Some(value) => Poll::Ready(GeneratorState::Yield(value)),
                None => Poll::Pending,
            },
            Poll::Ready(value) => Poll::Ready(GeneratorState::Return(value)),
        }
    }
}

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

pub struct Resume<'g, A, G> {
    gen: Pin<&'g mut G>,
    arg: Option<A>,
}

impl<'g, A, G> Future for Resume<'g, A, G>
where
    G: AsyncGenerator<A>,
{
    type Output = GeneratorState<G::Yield, G::Return>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        this.gen.as_mut().poll_resume(cx, this.arg.take())
    }
}

impl<'g, A, G> Unpin for Resume<'g, A, G> {}
