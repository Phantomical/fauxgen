#![cfg_attr(nightly, feature(waker_getters))]

mod asynk;
mod common;
mod impls;
mod noop;
mod sync;
mod util;
mod waker;

pub use fakerator_macros::generator;

pub use crate::asynk::{AsyncGenerator, AsyncGeneratorExt, GenStream, Resume};
pub use crate::sync::{GenIter, Generator, GeneratorExt};

type TokenId = *const ();

/// Copied from std.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GeneratorState<Y, R> {
    Yield(Y),
    Return(R),
}

#[doc(hidden)]
pub mod detail {
    pub use crate::asynk::GeneratorWrapper as AsyncGeneratorWrapper;
    pub use crate::common::{GeneratorToken, YieldFuture};
    pub use crate::sync::GeneratorWrapper as SyncGeneratorWrapper;
}
