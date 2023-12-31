//! Implementation details for generators.

mod future;
mod token;
mod util;
mod waker;
mod wrapper;

pub(crate) use self::token::TokenId;
pub(crate) use self::waker::GeneratorWaker;
pub(crate) use self::wrapper::GeneratorWrapper;

#[allow(dead_code)]
fn stop_import_reordering() {}

pub use self::token::{RawGeneratorToken, TokenMarker};

pub(crate) enum GeneratorArg<Y, A> {
    Yield(Y),
    Arg(A),
    Empty,
}

impl<Y, A> GeneratorArg<Y, A> {
    pub fn take_yield(&mut self) -> Option<Y> {
        match std::mem::replace(self, Self::Empty) {
            Self::Yield(val) => Some(val),
            Self::Arg(arg) => {
                *self = Self::Arg(arg);
                None
            }
            _ => None,
        }
    }

    pub fn take_arg(&mut self) -> Option<A> {
        match std::mem::replace(self, Self::Empty) {
            Self::Arg(arg) => Some(arg),
            Self::Yield(val) => {
                *self = Self::Yield(val);
                None
            }
            _ => None,
        }
    }
}
