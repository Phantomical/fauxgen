#![cfg_attr(nightly, feature(waker_getters))]
#![cfg_attr(feature = "std-generators", feature(generator_trait))]

#[cfg(not(feature = "std-generators"))]
mod core;

#[cfg(feature = "std-generators")]
mod core {
    pub use std::ops::{Generator, GeneratorState};
}

#[path = "async.rs"]
mod asynk;
mod detail;
mod impls;
mod stream;
mod token;
mod iter;

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

    pub async fn register<'t, Y, A>(
        token: Pin<&'t RawGeneratorToken<Y, A>>,
    ) -> GeneratorToken<'t, Y, A> {
        unsafe {
            // SAFETY: register is only called from the prelude generated by the
            //         #[generator] macro. The macro takes responsibility for ensuring that
            //         the parameters match.
            token.as_ref().register().await;

            // SAFETY: We just registered the token on the line above.
            GeneratorToken::new(token)
        }
    }
}

pub use fakerator_macros::generator;

pub use crate::asynk::{AsyncGenerator, Resume};
pub use crate::iter::GeneratorIter;
pub use crate::core::{Generator, GeneratorState};
pub use crate::stream::GeneratorStream;
pub use crate::token::GeneratorToken;
