use std::future::Future;
use std::pin::Pin;

use crate::detail::GeneratorWrapper;
use crate::{Generator, GeneratorState};

pub struct SyncGenerator<F, Y, A>(GeneratorWrapper<F, Y, A>);

impl<F, Y, A> SyncGenerator<F, Y, A> {
    pub(crate) fn new(future: F) -> Self {
        Self(GeneratorWrapper::new(future))
    }
}

impl<F, Y, A> Generator<A> for SyncGenerator<F, Y, A>
where
    F: Future,
{
    type Yield = Y;
    type Return = F::Output;

    fn resume(self: Pin<&mut Self>, arg: A) -> crate::GeneratorState<Self::Yield, Self::Return> {
        let wrapper = unsafe { self.map_unchecked_mut(|this| &mut this.0) };
        wrapper.resume(arg)
    }
}

impl<F, Y, A> Unpin for SyncGenerator<F, Y, A> where F: Unpin {}

impl<F, Y> Iterator for SyncGenerator<F, Y, ()>
where
    F: Future<Output = ()>,
    Self: Unpin,
{
    type Item = <Self as Generator>::Yield;

    fn next(&mut self) -> Option<Self::Item> {
        Pin::new(self).next()
    }
}

impl<F, Y> Iterator for Pin<&mut SyncGenerator<F, Y, ()>>
where
    F: Future<Output = ()>,
{
    type Item = <Self as Generator>::Yield;

    fn next(&mut self) -> Option<Self::Item> {
        match self.as_mut().resume(()) {
            GeneratorState::Yielded(value) => Some(value),
            GeneratorState::Complete(()) => None,
        }
    }
}

impl<F, Y> Iterator for Pin<Box<SyncGenerator<F, Y, ()>>>
where
    F: Future<Output = ()>,
{
    type Item = <Self as Generator>::Yield;

    fn next(&mut self) -> Option<Self::Item> {
        self.as_mut().next()
    }
}
