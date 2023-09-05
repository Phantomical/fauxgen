use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::detail::future::with_context;
use crate::detail::waker::GeneratorWaker;
use crate::detail::GeneratorArg;
use crate::export::{AsyncGenerator, SyncGenerator};
use crate::GeneratorToken;

used_in_docs!(SyncGenerator, AsyncGenerator, GeneratorToken);

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) struct TokenId(*const ());

impl TokenId {
    pub fn invalid() -> Self {
        Self(std::ptr::null())
    }

    fn new(value: *const ()) -> Self {
        Self(value)
    }

    pub fn is_valid(self) -> bool {
        !self.0.is_null()
    }
}

unsafe impl Send for TokenId {}
unsafe impl Sync for TokenId {}

/// Marker type used to ensure that the generator wrapper and the generator
/// token share the same yield and argument types.
///
/// When using the [`gen!`] macro we need a way to ensure that the
/// [`SyncGenerator`] or [`AsyncGenerator`] types have the same type parameters
/// as the [`GeneratorToken`] passed to the user code. We can't pass the token
/// itself, since it is needed elsewhere, so instead we used this zero-sized
/// marker type to bridge the gap.
///
/// [`gen!`]: crate::gen!
pub struct TokenMarker<Y, A>(PhantomData<(Y, A)>);

impl<Y, A> TokenMarker<Y, A> {
    pub const fn new() -> Self {
        Self(PhantomData)
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

    /// Returns a [`TokenMarker`] that shares the same `Y` and `A` parameters as
    /// this `RawGeneratorToken`.
    pub fn marker(&self) -> TokenMarker<Y, A> {
        TokenMarker::new()
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
