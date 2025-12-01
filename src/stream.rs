use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::Stream;

use crate::{AsyncGenerator, GeneratorState};

#[cfg(feature = "macros")]
use crate::generator;

#[cfg(feature = "macros")]
used_in_docs!(generator);

/// Wrapper around an async generator that implements [`Stream`].
///
/// The generators created by the [`generator`] macro implement [`Stream`] by
/// default. However, other implementations of [`AsyncGenerator`] will need this
/// wrapper type in order to be used as a stream.
pub struct GeneratorStream<G>(G);

impl<G> GeneratorStream<G> {
    pub fn new(gen: G) -> Self {
        Self(gen)
    }

    pub fn into_inner(self) -> G {
        self.0
    }
}

impl<G> Stream for GeneratorStream<G>
where
    G: AsyncGenerator<(), Return = ()>,
{
    type Item = G::Yield;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let gen = unsafe { self.map_unchecked_mut(|s| &mut s.0) };
        gen.poll_resume(cx, Some(())).map(|state| match state {
            GeneratorState::Yielded(value) => Some(value),
            GeneratorState::Complete(()) => None,
        })
    }
}

/// Wrapper around a generator that yields values and returns a result.
///
/// Often when working with streams you end up with a stream of results where
/// you want to abort things after the first error. By using a generator with
/// a result return type you can use `?` within the generator itself and still
/// get a stream which is usable.
///
/// This wrapper type wraps a generator that yields a series of values and
/// returns a result. All the yielded values become `Ok(v)` values in the stream
/// and returning an error emits a final `Err(e)` value before the stream
/// completes.
///
/// # Example
/// This stream will yield `Ok(44)`, `Ok(88)`, `Err("ran out of numbers")` and
/// then finish:
/// ```
/// use fauxgen::GeneratorTryStream;
///
/// #[fauxgen::generator(yield = i32)]
/// fn my_stream() -> Result<(), &'static str> {
///     r#yield!(44);
///     r#yield!(88);
///     Err("ran out of numbers")
/// }
///
/// let stream = GeneratorTryStream::new(my_stream());
/// ```
pub struct GeneratorTryStream<G> {
    gen: G,
    done: bool,
}

impl<G> GeneratorTryStream<G> {
    /// Create a stream from an existing generator.
    pub fn new(gen: G) -> Self {
        Self { gen, done: false }
    }

    /// Convert this stream back into the generator.
    pub fn into_inner(self) -> G {
        self.gen
    }
}

impl<G, E> Stream for GeneratorTryStream<G>
where
    G: AsyncGenerator<(), Return = Result<(), E>>,
{
    type Item = Result<G::Yield, E>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // SAFETY: This is just pin projection so it is safe.
        let (gen, done) = unsafe {
            let this = self.get_unchecked_mut();
            (Pin::new_unchecked(&mut this.gen), &mut this.done)
        };

        if *done {
            return Poll::Ready(None);
        }

        match gen.poll_resume(cx, Some(())) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(GeneratorState::Yielded(item)) => Poll::Ready(Some(Ok(item))),
            Poll::Ready(GeneratorState::Complete(result)) => {
                *done = true;

                match result {
                    Ok(()) => Poll::Ready(None),
                    Err(e) => Poll::Ready(Some(Err(e))),
                }
            }
        }
    }
}
