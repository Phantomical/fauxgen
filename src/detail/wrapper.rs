use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

use crate::detail::{GeneratorArg, GeneratorWaker, TokenId};
use crate::GeneratorState;

pub(crate) struct GeneratorWrapper<F, Y, A> {
    future: F,
    token: TokenId,
    _marker: PhantomData<(Y, A)>,
}

impl<F, Y, A> GeneratorWrapper<F, Y, A> {
    pub fn new(future: F) -> Self {
        Self {
            future,
            token: TokenId::invalid(),
            _marker: PhantomData,
        }
    }
}

impl<F, Y, A, R> GeneratorWrapper<F, Y, A>
where
    F: Future<Output = R>,
{
    fn poll(
        self: Pin<&mut Self>,
        waker: Option<&Waker>,
        arg: &mut GeneratorArg<Y, A>,
    ) -> Poll<GeneratorState<Y, R>> {
        let this = unsafe { self.get_unchecked_mut() };
        let future = unsafe { Pin::new_unchecked(&mut this.future) };

        // SAFETY: GeneratorWaker's clone impl returns a different waker so it will not
        //         outlive this function. This ensures that it will not outlive the
        //         references passed in here.
        let waker = unsafe { GeneratorWaker::new(waker, arg, &mut this.token) };
        let waker = std::pin::pin!(waker);

        // SAFETY: waker will not outlive this function.
        let waker = unsafe { waker.as_ref().to_waker() };
        let mut context = Context::from_waker(&waker);
        match future.poll(&mut context) {
            Poll::Pending => match arg.take_yield() {
                Some(value) => Poll::Ready(GeneratorState::Yield(value)),
                None => Poll::Pending,
            },
            Poll::Ready(value) => Poll::Ready(GeneratorState::Return(value)),
        }
    }

    /// Resume the generator and run it until it yields a value or returns.
    ///
    /// # Panics
    /// Panics if the internal generator function returns pending without having
    /// yielded a value.
    pub fn resume(self: Pin<&mut Self>, arg: A) -> GeneratorState<Y, R> {
        let mut arg = GeneratorArg::Arg(arg);
        match self.poll(None, &mut arg) {
            Poll::Pending => panic!("generator function returned pending without yielding a value"),
            Poll::Ready(state) => state,
        }
    }

    /// Resume the generator and run it until the next await point.
    ///
    /// This can be any of a yield point, a return, or an internal future being
    /// used.
    pub fn poll_resume(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        arg: &mut GeneratorArg<Y, A>,
    ) -> Poll<GeneratorState<Y, R>> {
        self.poll(Some(cx.waker()), arg)
    }
}

impl<F, Y, A> Unpin for GeneratorWrapper<F, Y, A> where F: Unpin {}
