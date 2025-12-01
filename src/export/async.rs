use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::Stream;

use crate::detail::{GeneratorArg, GeneratorWrapper};
use crate::{AsyncGenerator as _, GeneratorState, Resume};

#[must_use = "generators are lazy and do nothing unless polled"]
pub struct AsyncGenerator<F, Y, A> {
    inner: GeneratorWrapper<F, Y, A>,
    arg: GeneratorArg<Y, A>,
}

impl<F, Y, A> AsyncGenerator<F, Y, A> {
    pub(crate) fn new(future: F) -> Self {
        Self {
            inner: GeneratorWrapper::new(future),
            arg: GeneratorArg::Empty,
        }
    }
}

impl<F, Y, A> AsyncGenerator<F, Y, A>
where
    F: Future,
{
    pub fn resume(self: Pin<&mut Self>, arg: A) -> Resume<'_, A, Self> {
        <Self as crate::AsyncGenerator<A>>::resume(self, arg)
    }
}

impl<F, Y, A> crate::AsyncGenerator<A> for AsyncGenerator<F, Y, A>
where
    F: Future,
{
    type Yield = Y;
    type Return = F::Output;

    fn poll_resume(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        arg: Option<A>,
    ) -> Poll<GeneratorState<Self::Yield, Self::Return>> {
        let this = unsafe { self.get_unchecked_mut() };
        let wrapper = unsafe { Pin::new_unchecked(&mut this.inner) };

        if let Some(arg) = arg {
            this.arg = GeneratorArg::Arg(arg);
        }

        wrapper.poll_resume(cx, &mut this.arg)
    }
}

impl<F, Y, A> Unpin for AsyncGenerator<F, Y, A> where F: Unpin {}

impl<F, Y> Stream for AsyncGenerator<F, Y, ()>
where
    F: Future<Output = ()>,
{
    type Item = Y;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.poll_resume(cx, Some(())).map(|state| match state {
            GeneratorState::Yielded(value) => Some(value),
            GeneratorState::Complete(()) => None,
        })
    }
}

unsafe impl<F: Send, Y, A> Send for AsyncGenerator<F, Y, A> {}

// SAFETY: We only expose &mut methods so the generator can never be accessed
//         concurrently.
unsafe impl<F: Send, Y, A> Sync for AsyncGenerator<F, Y, A> {}
