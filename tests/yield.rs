#![cfg(nightly)]
#![feature(generators)]

// This needs to be in an inner module so that rustc doesn't error on the yield
// syntax.
#[cfg(nightly)]
#[path = "yield/inner.rs"]
mod inner;
