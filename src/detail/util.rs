use std::task::{RawWaker, RawWakerVTable, Waker};

pub(crate) struct RawWakerParts {
    pub data: *const (),
    pub vtable: *const RawWakerVTable,
}

#[cfg(not(nightly))]
pub(crate) fn waker_as_raw(waker: &Waker) -> &RawWaker {
    assert_eq!(
        std::mem::size_of::<Waker>(),
        std::mem::size_of::<RawWaker>()
    );

    // SAFETY: Waker is annotated with `#[repr(transparent)]` so this is currently
    //         safe. It is not a stable guarantee though and we should use
    //         Waker::as_raw once waker_getters stablilizes.
    unsafe { &*(waker as *const Waker as *const RawWaker) }
}

#[cfg(nightly)]
pub(crate) fn waker_as_raw(waker: &Waker) -> &RawWaker {
    waker.as_raw()
}

pub(crate) fn waker_into_raw(waker: Waker) -> RawWaker {
    unsafe { std::mem::transmute(waker) }
}

#[cfg(not(nightly))]
pub(crate) fn waker_into_parts(waker: &RawWaker) -> RawWakerParts {
    static TEST_VTABLE: RawWakerVTable =
        RawWakerVTable::new(crate::noop::noop_clone, drop, drop, drop);

    fn do_transmute(waker: &RawWaker) -> RawWakerParts {
        assert_eq!(
            std::mem::size_of_val(waker),
            std::mem::size_of::<[*const (); 2]>()
        );

        unsafe { std::ptr::read(waker as *const RawWaker as *const _) }
    }

    fn assert_transmute_ok() {
        let waker = RawWaker::new(std::ptr::null(), &TEST_VTABLE);
        let parts = do_transmute(&waker);

        assert_eq!(parts.vtable, &TEST_VTABLE);
        assert!(parts.data.is_null());
    }

    assert_transmute_ok();
    do_transmute(waker)
}

#[cfg(nightly)]
pub(crate) fn waker_into_parts(waker: &RawWaker) -> RawWakerParts {
    RawWakerParts {
        data: waker.data(),
        vtable: waker.vtable(),
    }
}
