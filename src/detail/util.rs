use std::task::{RawWaker, Waker};

pub(crate) fn waker_into_raw(waker: Waker) -> RawWaker {
    // SAFETY: Waker is annotated with `#[repr(transparent)]` so this is currently
    //         safe. It is not a stable guarantee though so it may break in the
    //         future.
    //
    // Ideally the waker_getters feature would include a function to do this
    // before it stabilizes.
    unsafe { std::mem::transmute(waker) }
}
