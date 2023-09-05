use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::common::GeneratorArg;
use crate::waker::GeneratorWaker;
use crate::GeneratorState;

/// The generator trait, copied from std.
pub trait Generator<A = ()> {
    type Yield;
    type Return;

    fn resume(self: Pin<&mut Self>, arg: A) -> GeneratorState<Self::Yield, Self::Return>;
}

pub trait GeneratorExt<A = ()>: Generator<A> {
    fn iter(self) -> GenIter<Self>
    where
        Self: Unpin + Sized,
    {
        GenIter(self)
    }
}

impl<A, G> GeneratorExt<A> for G where G: Generator<A> {}

pub struct GeneratorWrapper<F, Y, A> {
    future: F,
    _arg: PhantomData<(Y, A)>,
}

impl<F, Y, A> GeneratorWrapper<F, Y, A> {
    pub fn new(future: F) -> Self {
        Self {
            future,
            _arg: PhantomData,
        }
    }
}

impl<F, Y, A> Generator<A> for GeneratorWrapper<F, Y, A>
where
    F: Future,
{
    type Yield = Y;
    type Return = F::Output;

    fn resume(self: Pin<&mut Self>, arg: A) -> GeneratorState<Self::Yield, Self::Return> {
        let mut arg: GeneratorArg<Y, A> = GeneratorArg::Arg(arg);

        // SAFETY: GeneratorWaker's clone impl returns a different waker so none of the
        //         references stored in waker will outlive this function.
        let waker = unsafe { GeneratorWaker::new(None, &mut arg) };

        // SAFETY: waker will not outlive this function.
        let waker = unsafe { waker.to_waker() };

        // SAFETY: We don't move anything using the mut reference so this is safe.
        let future = unsafe { self.map_unchecked_mut(|this| &mut this.future) };

        let mut context = Context::from_waker(&waker);
        match future.poll(&mut context) {
            Poll::Pending => match arg.take_yield() {
                Some(value) => GeneratorState::Yield(value),
                None => panic!("generator yielded without producing a value"),
            },
            Poll::Ready(value) => GeneratorState::Return(value),
        }
    }
}

impl<F, Y, A> Unpin for GeneratorWrapper<F, Y, A> where F: Unpin {}

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
