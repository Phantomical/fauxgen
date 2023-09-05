use std::task::{RawWaker, RawWakerVTable};

static NOOP_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(noop_clone, drop, drop, drop);

pub(crate) unsafe fn noop_clone(_: *const ()) -> RawWaker {
    raw_waker()
}

pub(crate) fn raw_waker() -> RawWaker {
    RawWaker::new(std::ptr::null(), &NOOP_WAKER_VTABLE)
}
