use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::detail::future::with_context;
use crate::detail::waker::GeneratorWaker;
use crate::detail::GeneratorArg;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) struct TokenId(*const ());

impl TokenId {
    pub fn invalid() -> Self {
        Self(std::ptr::null())
    }

    // This shoud
    fn new(value: *const ()) -> Self {
        Self(value)
    }

    pub fn is_valid(self) -> bool {
        !self.0.is_null()
    }
}

pub struct RawGeneratorToken<Y, A> {
    // In order for this crate to guarantee correctness we need to ensure that each
    // `GeneratorToken` has a unique address. However, the token doesn't actually store any data so
    // we have a dummy u8 variable here to ensure it is not zero-sized.
    _filler: u8,
    _marker: PhantomData<(Y, A)>,
}

impl<Y, A> RawGeneratorToken<Y, A> {
    pub(crate) fn new() -> Self {
        Self {
            _filler: 0,
            _marker: PhantomData,
        }
    }

    /// Returns the unique [`TokenId`] for this `GeneratorToken`.
    ///
    /// The token id will be unique for this generator as long as this token is
    /// still alive.
    pub(crate) fn id(self: Pin<&Self>) -> TokenId {
        TokenId::new(self.get_ref() as *const _ as _)
    }

    /// Register this token with the current generator.
    ///
    /// This is used to ensure that the values yielded to the generator have the
    /// type the generator expects. It will be automatically called by generated
    /// generator prelude.
    ///
    /// # Safety
    /// The `Y` and `A` parameters on this `GeneratorToken` _must_ match those
    /// on the `GeneratorArg<Y, A>` instance stored within the waker.
    ///
    /// # Panics
    /// - Panics if the waker for this function is not a [`GeneratorWaker`]
    /// - Panics if the waker already has a token registered.
    pub async unsafe fn register(self: Pin<&Self>) {
        with_context(|cx| {
            let waker = match GeneratorWaker::from_waker_ref(cx.waker()) {
                Some(waker) => waker,
                None => panic!("called GeneratorToken::register with unsupported waker"),
            };

            waker.set_id(self.id());
        })
        .await
    }

    /// Yield a value from the current generator.
    pub async fn yield_(self: Pin<&Self>, value: Y) -> A {
        YieldFuture::new(value, self).await
    }

    pub async fn argument(self: Pin<&Self>) -> A {
        with_context(|cx| {
            let waker = match GeneratorWaker::from_waker_ref(cx.waker()) {
                Some(waker) => waker,
                None => panic!("called GeneratorToken::arg with unsupported waker"),
            };

            let arg = unsafe { &mut *waker.arg_raw(self) };
            arg.take_arg().expect("no argument present when resuming")
        })
        .await
    }
}

struct YieldFuture<'t, Y, A> {
    value: Option<Y>,
    token: Pin<&'t RawGeneratorToken<Y, A>>,
}

impl<'t, Y, A> YieldFuture<'t, Y, A> {
    fn new(value: Y, token: Pin<&'t RawGeneratorToken<Y, A>>) -> Self {
        Self {
            value: Some(value),
            token,
        }
    }
}

impl<Y, A> Future for YieldFuture<'_, Y, A> {
    type Output = A;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let waker = GeneratorWaker::from_waker_ref(cx.waker())
            .expect("called GeneratorToken::yield with unsupported waker");

        let arg = unsafe { &mut *waker.arg_raw(self.token.as_ref()) };

        match self.value.take() {
            Some(value) => {
                *arg = GeneratorArg::Yield(value);
                Poll::Pending
            }
            None => Poll::Ready(arg.take_arg().expect("no argument present when resuming")),
        }
    }
}

impl<Y, A> Unpin for YieldFuture<'_, Y, A> {}
