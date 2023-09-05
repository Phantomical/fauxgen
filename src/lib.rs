#![cfg_attr(nightly, feature(waker_getters))]

use std::ops::DerefMut;
use std::panic::AssertUnwindSafe;
use std::pin::Pin;

mod common;
mod noop;
mod sync;

pub use genawaiter_macros::generator;

/// The generator trait, copied from std.
pub trait Generator<A = ()> {
    type Yield;
    type Return;

    fn resume(self: Pin<&mut Self>, arg: A) -> GeneratorState<Self::Yield, Self::Return>;

    fn iter(self) -> GenIter<Self>
    where
        Self: Unpin + Sized,
    {
        GenIter(self)
    }
}

/// Copied from std.
pub enum GeneratorState<Y, R> {
    Yield(Y),
    Return(R),
}

impl<A, G> Generator<A> for &mut G
where
    G: Generator<A> + Unpin + ?Sized,
{
    type Yield = G::Yield;
    type Return = G::Return;

    fn resume(mut self: Pin<&mut Self>, arg: A) -> GeneratorState<Self::Yield, Self::Return> {
        Pin::new(&mut **self).resume(arg)
    }
}

impl<A, G> Generator<A> for AssertUnwindSafe<G>
where
    G: Generator<A>,
{
    type Yield = G::Yield;
    type Return = G::Return;

    fn resume(self: Pin<&mut Self>, arg: A) -> GeneratorState<Self::Yield, Self::Return> {
        let gen = unsafe { self.map_unchecked_mut(|this| &mut **this) };
        gen.resume(arg)
    }
}

impl<A, G: Generator<A>> Generator<A> for Box<G> {
    type Yield = G::Yield;
    type Return = G::Return;

    fn resume(self: Pin<&mut Self>, arg: A) -> GeneratorState<Self::Yield, Self::Return> {
        let gen = unsafe { self.map_unchecked_mut(|this| &mut **this) };
        gen.resume(arg)
    }
}

impl<A, P> Generator<A> for Pin<P>
where
    P: DerefMut,
    P::Target: Generator<A>,
{
    type Yield = <P::Target as Generator<A>>::Yield;
    type Return = <P::Target as Generator<A>>::Return;

    fn resume(self: Pin<&mut Self>, arg: A) -> GeneratorState<Self::Yield, Self::Return> {
        let target = unsafe { self.get_unchecked_mut().as_mut() };
        <P::Target as Generator<A>>::resume(target, arg)
    }
}

pub struct GenIter<G>(G);

impl<G> Iterator for GenIter<G>
where
    G: Generator<(), Return = ()> + Unpin,
{
    type Item = G::Yield;

    fn next(&mut self) -> Option<Self::Item> {
        match Pin::new(&mut self.0).resume(()) {
            GeneratorState::Return(()) => None,
            GeneratorState::Yield(value) => Some(value),
        }
    }
}

#[doc(hidden)]
pub mod detail {
    pub use crate::common::{GeneratorToken, YieldFuture};
    pub use crate::sync::GeneratorWrapper as SyncGeneratorWrapper;
}
