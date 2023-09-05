use std::pin::Pin;

use crate::detail::RawGeneratorToken;

/// A generator token ties together the executor and the generator itself.
///
/// It is what allows us to yield values back out of the generator.
pub struct GeneratorToken<'t, Y, A>(Pin<&'t RawGeneratorToken<Y, A>>);

impl<'t, Y, A> GeneratorToken<'t, Y, A> {
    /// Create this token from a pinned raw token reference.
    ///
    /// # Safety
    /// `token` must have already been registered in the current generator
    /// context.
    pub(crate) unsafe fn new(token: Pin<&'t RawGeneratorToken<Y, A>>) -> Self {
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
}

impl<Y, A> Copy for GeneratorToken<'_, Y, A> {}
impl<Y, A> Clone for GeneratorToken<'_, Y, A> {
    fn clone(&self) -> Self {
        *self
    }
}
