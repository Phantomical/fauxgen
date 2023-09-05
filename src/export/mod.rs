//! Exported types used by the `#[generator]` macro.
//!
//! These are public but are not part of the public API.

#[path = "async.rs"]
mod asynk;
mod sync;

pub use self::asynk::AsyncGenerator;
pub use self::sync::SyncGenerator;
