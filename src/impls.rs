use std::ops::DerefMut;
use std::panic::AssertUnwindSafe;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::{AsyncGenerator, GeneratorState};

impl<A, G> AsyncGenerator<A> for &mut G
where
    G: AsyncGenerator<A> + Unpin + ?Sized,
{
    type Return = G::Return;
    type Yield = G::Yield;

    fn poll_resume(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        arg: Option<A>,
    ) -> Poll<GeneratorState<Self::Yield, Self::Return>> {
        Pin::new(&mut **self).poll_resume(cx, arg)
    }
}

impl<A, G> AsyncGenerator<A> for AssertUnwindSafe<G>
where
    G: AsyncGenerator<A>,
{
    type Yield = G::Yield;
    type Return = G::Return;

    fn poll_resume(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        arg: Option<A>,
    ) -> Poll<GeneratorState<Self::Yield, Self::Return>> {
        let gen = unsafe { self.map_unchecked_mut(|this| &mut **this) };
        gen.poll_resume(cx, arg)
    }
}

impl<A, G> AsyncGenerator<A> for Box<G>
where
    G: AsyncGenerator<A>,
{
    type Yield = G::Yield;
    type Return = G::Return;

    fn poll_resume(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        arg: Option<A>,
    ) -> Poll<GeneratorState<Self::Yield, Self::Return>> {
        let gen = unsafe { self.map_unchecked_mut(|this| &mut **this) };
        gen.poll_resume(cx, arg)
    }
}

impl<A, P> AsyncGenerator<A> for Pin<P>
where
    P: DerefMut,
    P::Target: AsyncGenerator<A>,
{
    type Yield = <P::Target as AsyncGenerator<A>>::Yield;
    type Return = <P::Target as AsyncGenerator<A>>::Return;

    fn poll_resume(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        arg: Option<A>,
    ) -> Poll<GeneratorState<Self::Yield, Self::Return>> {
        let target = unsafe { self.get_unchecked_mut().as_mut() };
        target.poll_resume(cx, arg)
    }
}
