#![cfg_attr(nightly, feature(waker_getters))]

use std::pin::Pin;

use common::GeneratorToken;

mod common;
mod noop;
mod sync;

/// The generator trait, copied from std.
pub trait Generator<A = ()> {
    type Yield;
    type Return;

    fn resume(self: Pin<&mut Self>, arg: A) -> GeneratorState<Self::Yield, Self::Return>;
}

/// Copied from std.
pub enum GeneratorState<Y, R> {
    Yield(Y),
    Return(R),
}

/// Macro used for yielding
#[doc(hidden)]
#[macro_export]
macro_rules! r#yield {
    ($token:expr, $value:expr) => {{
        let value = $value;
        unsafe { $token.do_yield(value).await }
    }};
}

pub fn my_generator(mut start: u64) -> impl Generator<Yield = u64, Return = ()> {
    let __token: GeneratorToken<u64, ()> = crate::detail::GeneratorToken::new();

    macro_rules! r#yield {
        ($value:expr) => {{
            __token.do_yield($value);
        }};
    }
    crate::detail::SyncGeneratorWrapper::new(async move {
        while let Some(value) = start.checked_shl(1) {
            start = value;
            r#yield!(value);
        }
    })
}

#[doc(hidden)]
pub mod detail {
    pub use crate::common::{GeneratorToken, YieldFuture};
    pub use crate::sync::GeneratorWrapper as SyncGeneratorWrapper;
}

#[cfg(not(all()))]
mod dummy {
    #[generator(yield u64)]
    pub fn my_generator(mut start: u64) {
        while let Some(value) = start.checked_shl(1) {
            start = value;
            // yield value;
        }
    }

    // becomes ...
}
