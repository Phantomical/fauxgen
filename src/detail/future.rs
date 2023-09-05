use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub(crate) fn with_context<F, O>(func: F) -> WithContext<F>
where
    F: FnOnce(&mut Context) -> O,
{
    WithContext::new(func)
}

/// Helper future for running a function with access to the [`Context`] the
/// current future is being polled with.
pub(crate) struct WithContext<F>(Option<F>);

impl<F, O> WithContext<F>
where
    F: FnOnce(&mut Context) -> O,
{
    pub fn new(func: F) -> Self {
        Self(Some(func))
    }
}

impl<F, O> Future for WithContext<F>
where
    F: FnOnce(&mut Context) -> O,
{
    type Output = O;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let func = self.0.take().expect("future has already completed");
        Poll::Ready(func(cx))
    }
}

impl<F> Unpin for WithContext<F> {}
