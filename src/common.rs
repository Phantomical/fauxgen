use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

pub(crate) static GENERATOR_WAKER_VTABLE: RawWakerVTable =
    RawWakerVTable::new(crate::noop::noop_clone, drop, drop, drop);

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

pub struct GeneratorToken<Y, A>(PhantomData<(Y, A)>);

impl<Y, A> GeneratorToken<Y, A> {
    #[doc(hidden)]
    pub fn new() -> Self {
        Self(PhantomData)
    }

    pub fn do_yield(&self, value: Y) -> YieldFuture<Y, A> {
        unsafe { YieldFuture::new(value) }
    }
}

impl<Y, A> Unpin for GeneratorToken<Y, A> {}

pub struct YieldFuture<Y, A> {
    value: Option<Y>,
    _arg: PhantomData<A>,
}

impl<Y, A> YieldFuture<Y, A> {
    /// # Safety
    /// This requires that the outer `GeneratorWrapper` type has the exact same
    /// `Y` and `A` parameters.
    pub unsafe fn new(value: Y) -> Self {
        Self {
            value: Some(value),
            _arg: PhantomData,
        }
    }
}

#[allow(dead_code)]
struct RawFaker {
    data: *const (),
    vtable: *const RawWakerVTable,
}

impl<Y, A> Future for YieldFuture<Y, A> {
    type Output = A;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let waker = cx.waker();

        #[cfg(not(nightly))]
        let arg: &mut GeneratorArg<Y, A> = {
            // SAFETY: Waker is annotated with `#[repr(transparent)]` so this is currently
            //         safe. It is not a stable guarantee though and we should use
            //         Waker::as_raw once waker_getters stablilizes.
            let waker = unsafe { &*(waker as *const Waker as *const RawWaker) };

            assert_eq!(
                std::mem::size_of_val(waker),
                std::mem::size_of::<RawFaker>()
            );

            // SAFETY: This is not safe. However, in cases where it doesn't work it will
            //         fail by panicking instead of going on to cause further UB
            //         (hopefully). That doesn't change the fact that transmuting rust
            //         structs like this is UB but it should at least limit the
            //         consequences.
            let faker: RawFaker =
                unsafe { std::ptr::read(waker as *const RawWaker as *const RawFaker) };

            if faker.vtable != &GENERATOR_WAKER_VTABLE as *const _ {
                panic!("YieldFuture called with an unsupported waker");
            }

            unsafe { &mut *(faker.data as *mut _) }
        };

        #[cfg(nightly)]
        let arg: &mut GeneratorArg<Y, A> = {
            let waker = waker.as_raw();

            if waker.vtable() as *const _ != &GENERATOR_WAKER_VTABLE as *const _ {
                panic!("YieldFuture called with an unsupported waker");
            }

            // SAFETY: We verified the vtable above so we know that the data pointer
            //         is a reference to a GeneratorArg instance. That the argument types
            //         are correct was asserted while constructing this future.
            unsafe { &mut *(waker.data() as *mut _) }
        };

        match self.value.take() {
            Some(value) => {
                *arg = GeneratorArg::Yield(value);
                Poll::Pending
            }
            None => match arg.take_arg() {
                Some(arg) => Poll::Ready(arg),
                None => panic!("no argument present when resuming"),
            },
        }
    }
}

impl<Y, A> Unpin for YieldFuture<Y, A> {}
