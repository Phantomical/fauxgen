#![cfg_attr(nightly, feature(waker_getters, generator_trait))]

use std::future::Future;

#[cfg(not(nightly))]
mod core;

#[cfg(nightly)]
mod core {
    pub use std::ops::{Generator, GeneratorState};
}

#[path = "async.rs"]
mod asynk;
mod detail;
mod impls;
mod iter;
mod stream;
mod token;

pub mod export;

#[doc(hidden)]
pub mod __private {
    use std::pin::Pin;

    use crate::export::*;
    use crate::GeneratorToken;

    // separate exports ..
    #[allow(dead_code)]
    fn _dummy() {}

    pub use std::future::Future;
    pub use std::pin::pin;

    pub use crate::detail::RawGeneratorToken;

    pub fn gen_sync<F, Y, A>(future: F) -> SyncGenerator<F, Y, A> {
        SyncGenerator::new(future)
    }

    pub fn gen_async<F, Y, A>(future: F) -> AsyncGenerator<F, Y, A> {
        AsyncGenerator::new(future)
    }

    pub fn token<Y, A>() -> RawGeneratorToken<Y, A> {
        RawGeneratorToken::new()
    }

    pub async fn register<Y, A>(token: Pin<&RawGeneratorToken<Y, A>>) -> GeneratorToken<Y, A> {
        unsafe {
            // SAFETY: register is only called from the prelude generated by the
            //         #[generator] macro. The macro takes responsibility for ensuring that
            //         the parameters match.
            GeneratorToken::register(token).await
        }
    }
}

pub use fauxgen_macros::generator;

pub use crate::asynk::{AsyncGenerator, Resume};
pub use crate::core::{Generator, GeneratorState};
pub use crate::iter::GeneratorIter;
pub use crate::stream::GeneratorStream;
pub use crate::token::GeneratorToken;

pub fn gen<Fn, Fut, Y, A>(
    func: Fn,
) -> export::SyncGenerator<impl Future<Output = Fut::Output>, Y, A>
where
    Fn: for<'t> FnOnce(GeneratorToken<'t, Y, A>) -> Fut,
    Fut: Future,
{
    export::SyncGenerator::new(async move {
        let token = std::pin::pin!(crate::detail::RawGeneratorToken::new());
        // SAFETY: We know the types match here because we control both sides of the
        //         generator context.
        let token: GeneratorToken<Y, A> = unsafe { GeneratorToken::register(token.as_ref()).await };

        func(token).await
    })
}

pub fn gen_async<Fn, Fut, Y, A>(
    func: Fn,
) -> export::AsyncGenerator<impl Future<Output = Fut::Output>, Y, A>
where
    Fn: for<'t> FnOnce(GeneratorToken<'t, Y, A>) -> Fut,
    Fut: Future,
{
    export::AsyncGenerator::new(async move {
        let token = std::pin::pin!(crate::detail::RawGeneratorToken::new());
        // SAFETY: We know the types match here because we control both sides of the
        //         generator context.
        let token: GeneratorToken<Y, A> = unsafe { GeneratorToken::register(token.as_ref()).await };

        func(token).await
    })
}
