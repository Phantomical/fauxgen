//! Generators on stable rust.

#![cfg_attr(nightly, feature(waker_getters))]
#![cfg_attr(feature = "std-generators", feature(generator_trait))]

extern crate self as fauxgen;

// A small helper macro to avoid unused_imports warnings for items that have
// only been pulled in to scope so they can be referred to in docs.
macro_rules! used_in_docs {
    ($( $name:ident ),+ $(,)? ) => {
        const _: () = {
            #[allow(unused_imports)]
            mod _used_in_docs {
                $( use super::$name; )+
            }
        };
    };
}

#[path = "async.rs"]
mod asynk;
mod detail;
mod impls;
mod iter;
mod stream;
mod token;

#[cfg(not(feature = "std-generators"))]
mod core;

#[cfg(feature = "std-generators")]
mod core {
    pub use std::ops::{Generator, GeneratorState};
}

pub mod export;

pub use fauxgen_macros::generator;

pub use crate::asynk::{AsyncGenerator, Resume};
pub use crate::core::{Generator, GeneratorState};
pub use crate::iter::GeneratorIter;
pub use crate::stream::GeneratorStream;
pub use crate::token::GeneratorToken;

#[macro_export]
macro_rules! gen {
    (async $func:expr) => { gen!(impl(gen_async) $func) };
    (      $func:expr) => { gen!(impl(gen_sync)  $func) };

    (impl($genfn:ident) $func:expr) => {{
        let func = $func;
        let token = $crate::__private::token();

        $crate::__private::$genfn(
            token.marker(),
            async move {
                let token: GeneratorToken<_, _> = $crate::__private::register_owned(token).await;
                func(token).await
            }
        )
    }}
}

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

    pub use crate::detail::{RawGeneratorToken, TokenMarker};

    pub fn gen_sync<F, Y, A>(_: TokenMarker<Y, A>, future: F) -> SyncGenerator<F, Y, A> {
        SyncGenerator::new(future)
    }

    pub fn gen_async<F, Y, A>(_: TokenMarker<Y, A>, future: F) -> AsyncGenerator<F, Y, A> {
        AsyncGenerator::new(future)
    }

    pub fn token<Y, A>() -> RawGeneratorToken<Y, A> {
        RawGeneratorToken::new()
    }

    pub async fn register<Y, A>(token: Pin<&RawGeneratorToken<Y, A>>) {
        // SAFETY: register is only called from the prelude generated by the
        //         #[generator] macro. The macro takes responsibility for ensuring that
        //         the parameters match.
        unsafe { token.register().await }
    }

    pub async fn register_owned<Y, A>(token: RawGeneratorToken<Y, A>) -> GeneratorToken<Y, A> {
        // SAFETY: register_owned is only called from the code emitted by the gen!
        //         macro. The macro takes responsibility for ensuring that the
        //         parameters match.
        unsafe { GeneratorToken::register(token).await }
    }
}
