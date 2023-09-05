use std::pin::Pin;

use crate::detail::RawGeneratorToken;

/// A generator token ties together the executor and the generator itself.
///
/// It is what allows us to yield values back out of the generator.
pub struct GeneratorToken<'t, Y, A>(Pin<&'t RawGeneratorToken<Y, A>>);

impl<'t, Y, A> GeneratorToken<'t, Y, A> {
    /// Create a new GeneratorToken by registering this one.
    ///
    /// # Safety
    /// The `Y` and `A` types for this token must mach those of the generator
    /// context.
    pub(crate) async unsafe fn register(
        token: Pin<&'t RawGeneratorToken<Y, A>>,
    ) -> GeneratorToken<'t, Y, A> {
        // SAFETY: The caller of this function ensures that the requirements here are
        //         upheld.
        token.register().await;

        Self(token)
    }

    /// Yield a value from this generator, returning control back to the caller.
    ///
    /// # Panics
    /// Panics if evaluated in the context of a generator other than the one
    /// this token was created for.
    pub async fn yield_(&self, value: Y) -> A {
        self.0.as_ref().yield_(value).await
    }

    /// Get the current argument without yielding.
    ///
    /// Normally [`yield_`] will yield a value and then read in the next
    /// argument. However, the very first argument passed in to the generator
    /// happens before the first call to [`yield_`]. This method is used to read
    /// that initial argument passed into the generator.
    ///
    /// # Panics
    /// - Panics if evaluated in the context of a generator other than the one
    ///   this token was created for.
    /// - Panics if there is no argument saved in the generator context. (e.g.
    ///   yield has already been called or argument was called multiple times)
    ///
    /// [`yield_`]: GeneratorToken::yield_
    pub async fn argument(&self) -> A {
        self.0.as_ref().argument().await
    }
}

impl<Y, A> Copy for GeneratorToken<'_, Y, A> {}
impl<Y, A> Clone for GeneratorToken<'_, Y, A> {
    fn clone(&self) -> Self {
        *self
    }
}
