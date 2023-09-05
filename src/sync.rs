use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll, RawWaker, Waker};

use crate::common::{GeneratorArg, GENERATOR_WAKER_VTABLE};
use crate::{Generator, GeneratorState};

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

        // SAFETY: We don't move anything using the mut reference so this is safe.
        let future = unsafe { self.map_unchecked_mut(|this| &mut this.future) };

        let raw_waker = RawWaker::new(&mut arg as *mut _ as *const (), &GENERATOR_WAKER_VTABLE);
        // SAFETY: the waker vtable ensures that we meet its preconditions.
        let waker = unsafe { Waker::from_raw(raw_waker) };
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
