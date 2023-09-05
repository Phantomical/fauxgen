//! This crate implements generators for Rust.
//!
//! Rust has built-in generators but they are currently unstable and so they can
//! only be used on nightly. This crate allows you to write your own generators
//! on stable rust by using async-await underneath the hood.
//!
//! # Defining a Generator
//! This crate provides two different ways to define generators. The first is as
//! a top-level function:
//!
//! ```
//! #[fauxgen::generator(yield = i32)]
//! fn generator() {
//!     r#yield!(1);
//!     r#yield!(2);
//! }
//! ```
//!
//! and the second is as a lambda using the [`gen!`] macro:
//!
//! ```
//! use fauxgen::{gen, GeneratorToken};
//!
//! let generator = fauxgen::gen!(|token: GeneratorToken<_>| async move {
//!     token.yield_(1i32).await;
//!     token.yield_(2i32).await;
//! });
//! ```
//!
//! In this case the generator uses a [`GeneratorToken`] instead of the `yield!`
//! macro.
//!
//! Generators can also be async
//! ```
//! use std::time::Duration;
//!
//! #[fauxgen::generator(yield = u32)]
//! async fn generator() {
//!     for i in 0u32..10 {
//!         tokio::time::sleep(Duration::from_millis(50)).await;
//!         r#yield!(i * 2);
//!     }
//! }
//! ```
//!
//! # Using a Generator
//! Generators all implement either [`Generator`] or [`AsyncGenerator`]. Simple
//! generators implement either [`Iterator`] or
//! [`Stream`](futures_core::Stream), depending on whether the generator is
//! async. This means that you can easily combine generators.
//!
//! Here we implement a generator that returns all the powers of two for a u32:
//! ```
//! #[fauxgen::generator(yield = u32)]
//! fn range(max: u32) {
//!     for i in 0..max {
//!         r#yield!(i);
//!     }
//! }
//!
//! #[fauxgen::generator(yield = u32)]
//! fn powers_of_two() {
//!     for i in std::pin::pin!(range(31)) {
//!         r#yield!(1 << i);
//!     }
//! }
//! ```
//!
//! Note that because `fauxgen` generators are actually rust futures under the
//! hood you will need to pin them before you can use them.
//!
//! # More Advanced Generator Usage
//! Most use cases for generators will likely involve using them as iterators or
//! streams. However, that is not all that they can do. In addition to the yield
//! parameter, generators have both
//! - an argument: which is the value passed in to `resume`, and,
//! - a return value: which is the value returned when the generator completes.
//!
//! A complete generator that uses all of these looks like this:
//! ```
//! use fauxgen::{GeneratorState, Generator};
//!
//! #[fauxgen::generator(yield = String, arg = u32)]
//! fn format_each() -> u64 {
//!     let mut count = 0;
//!     let mut value = 0;
//!
//!     while value < 100 {
//!         value = r#yield!(value.to_string());
//!         count += 1;
//!     }
//!
//!     count
//! }
//!
//! let mut gen = std::pin::pin!(format_each());
//!
//! for value in [0, 5, 10, 25, 125, 87, 31] {
//!     match gen.as_mut().resume(value) {
//!         GeneratorState::Yielded(text) => println!("{text}"),
//!         GeneratorState::Complete(count) => {
//!             println!("printed {count} items");
//!             break;
//!         }
//!     }
//! }
//! ```
//!
//! This is obviously somewhat harder to use than just using the generator as an
//! iterator but it does give you more abilities to use.
//!
//! ## Accessing the first argument
//! If you run the code above (or look closely) then you might notice that the
//! first argument passed into the generator is ignored. This usually isn't
//! what you want. In order to access the first argument you can use the
//! argument macro:
//!
//! ```
//! #[fauxgen::generator(yield = String, arg = u32)]
//! fn format_each() {
//!     let mut count = 0;
//!     let mut value = argument!();
//!
//!     while value < 100 {
//!         value = r#yield!(value.to_string());
//!         count += 1;
//!     }
//!
//!     count
//! }
//! ```
//!
//! Note that using the `argument!` macro after you have called `yield!` is
//! likely to result in a panic.

#![cfg_attr(nightly, feature(waker_getters))]
#![cfg_attr(std_generators, feature(generator_trait))]

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
mod export;
mod impls;
mod iter;
mod stream;
mod token;

#[cfg(not(std_generators))]
mod core;

#[cfg(std_generators)]
mod core {
    pub use std::ops::{Generator, GeneratorState};
}

#[cfg(feature = "macros")]
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
                let token: $crate::GeneratorToken<_, _> = $crate::__private::register_owned(token).await;
                func(token).await
            }
        )
    }}
}

#[doc(hidden)]
pub mod __private {
    use std::pin::Pin;

    use crate::GeneratorToken;

    // separate exports ..
    #[allow(dead_code)]
    fn _dummy() {}

    pub use std::future::Future;
    pub use std::pin::pin;

    pub use crate::detail::{RawGeneratorToken, TokenMarker};
    pub use crate::export::{AsyncGenerator, SyncGenerator};

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
