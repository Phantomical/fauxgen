use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::GeneratorState;

/// The trait implemented by asynchronous generators.
///
/// This trait works similar to [`Generator`] except that its [`resume`] method
/// is async.
///
/// [`Generator`]: crate::Generator
/// [`resume`]: AsyncGenerator::resume
pub trait AsyncGenerator<A = ()> {
    /// The type of value this generator yields.
    type Yield;

    /// The type of value this generator returns.
    type Return;

    /// Resume the execution of this generator.
    ///
    /// This function will resume execution of the generator or start execution
    /// if it hasn't already. This will return to the generator's last
    /// suspension point and resume execution from the last `yield` or `await`
    /// that the generator was at. The generator will continue executing until
    /// it either yields, returns, or awaits something, at which point this
    /// function will return.
    ///
    /// You likely want to use the [`resume`](AsyncGenerator::resume) function
    /// instead of this one.
    ///
    /// # Argument
    /// This function has an optional `arg` parameter to pass to the generator.
    /// In between yield points at least one call to `poll_resume` must have
    /// `arg` be `Some`. Should `arg` be `Some` multiple times between yield
    /// points then only one of the argument values will be kept and passed to
    /// the generator. The specific argument value that is passed may be any of
    /// the argument values passed since the last yield point.
    ///
    /// # Return value
    /// The [`GeneratorState`] returned by this function is in upon returning.
    /// If the `Yielded` variant is returned then the generator has reached a
    /// suspension point and the value has been yielded out. If `Complete` is
    /// returned then the generator has finished and attempting to resume it
    /// again will result in a panic.
    ///
    /// # Panics
    /// This function may panic if it is called after the `Complete` variant has
    /// been returned previously. Generators created by the `fauxgen` macros are
    /// guaranteed to panic in this case, custom implementations of
    /// `AsyncGenerator` are not required to do so.
    fn poll_resume(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        arg: Option<A>,
    ) -> Poll<GeneratorState<Self::Yield, Self::Return>>;

    /// Resume the execution of this generator.
    ///
    /// This method is an easier-to-use variant of [`poll_resume`] that returns
    /// a future that only resolves once the generator has either reached a
    /// suspension point or returned. This avoids all the difficulties around
    /// the proper time to pass in the generator argument.
    ///
    /// # Return value
    /// The [`GeneratorState`] enum yielded by the `Resume` future indicates
    /// what state the generator is in upon returning. If the `Yielded` variant
    /// is returned then the generator has reached a suspension point and a
    /// value has been yielded out. If `Complete` is returned then the generator
    /// has finished and attempting to resume the generator again may result in
    /// a panic.
    ///
    /// # Panics
    /// This function may panic if it is called after the `Complete` variant has
    /// been returned previously. Generators created by the `fauxgen` macros are
    /// guaranteed to panic in this case, custom implementations of
    /// `AsyncGenerator` are not required to do so.
    ///
    /// [`poll_resume`]: AsyncGenerator::poll_resume
    fn resume(self: Pin<&mut Self>, arg: A) -> Resume<'_, A, Self>
    where
        Self: Sized,
    {
        Resume {
            arg: Some(arg),
            gen: self,
        }
    }
}

/// A future used to implement [`AsyncGenerator::resume`].
///
/// See [`AsyncGenerator::resume`].
///
/// Note that dropping this future early without polling it to completion will
/// not result in any errors in the underlying generator. Calling [`resume`]
/// again will only result in another argument value being provided to the
/// generator.
///
/// [`resume`]: AsyncGenerator::resume
pub struct Resume<'g, A, G: ?Sized> {
    gen: Pin<&'g mut G>,
    arg: Option<A>,
}

impl<'g, A, G> Future for Resume<'g, A, G>
where
    G: AsyncGenerator<A> + ?Sized,
{
    type Output = GeneratorState<G::Yield, G::Return>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        this.gen.as_mut().poll_resume(cx, this.arg.take())
    }
}

impl<'g, A, G: ?Sized> Unpin for Resume<'g, A, G> {}
