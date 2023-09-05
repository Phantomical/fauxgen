use std::pin::Pin;
use std::ptr::NonNull;
use std::task::{RawWaker, RawWakerVTable, Waker};

use super::RawGeneratorToken;
use crate::detail::{GeneratorArg, TokenId};

pub(crate) struct GeneratorWaker {
    waker: Option<NonNull<Waker>>,
    id: *mut TokenId,
    arg: *mut (),
}

impl GeneratorWaker {
    /// Create a new `GeneratorWaker`
    ///
    /// # Safety
    /// - `waker` must remain valid until this `GeneratorWaker` instance is
    ///   dropped.
    /// - `arg` and `id` must remain valid while this `GeneratorWaker` instance
    ///   is being used to poll futures.
    pub unsafe fn new<Y, A>(
        waker: Option<&Waker>,
        arg: *mut GeneratorArg<Y, A>,
        id: *mut TokenId,
    ) -> Self {
        Self {
            waker: waker.map(NonNull::from),
            arg: arg as *mut (),
            id,
        }
    }

    /// Convert this `GeneratorWaker` reference to a [`Waker`].
    ///
    /// # Safety
    /// The `GeneratorWaker` reference must outlive the returned `Waker`.
    pub unsafe fn to_waker(self: Pin<&Self>) -> Waker {
        Waker::from_raw(RawWaker::new(
            self.get_ref() as *const _ as *const (),
            &GENERATOR_WAKER_VTABLE,
        ))
    }

    pub fn from_waker_ref(waker: &Waker) -> Option<&Self> {
        let waker = crate::util::waker_as_raw(waker);
        let parts = crate::util::waker_into_parts(waker);

        if parts.vtable != &GENERATOR_WAKER_VTABLE {
            return None;
        }

        // SAFETY: We validated the vtable above so we know this points to a
        //         GeneratorWaker instance.
        let waker = unsafe { &*(parts.data as *const Self) };

        Some(waker)
    }

    fn waker(&self) -> Option<&Waker> {
        unsafe { self.waker.map(|waker| waker.as_ref()) }
    }

    fn wake_by_ref(&self) {
        if let Some(waker) = self.waker() {
            waker.wake_by_ref()
        }
    }

    fn clone_waker(&self) -> RawWaker {
        match self.waker() {
            Some(waker) => crate::util::waker_into_raw(waker.clone()),
            None => noop_waker(),
        }
    }

    pub(super) fn set_id(&self, id: TokenId) {
        let waker_id = unsafe { &mut *self.id };

        // This ensures that it is only possible to register a token once.
        assert!(
            !waker_id.is_valid(),
            "this generator already has a registered token"
        );

        *waker_id = id;
    }

    /// Access the [`GeneratorArg`] pointer stored within this waker.
    ///
    /// # Panics
    /// Panics if `token` is not the token registered with this waker.
    pub fn arg_raw<Y, A>(&self, token: Pin<&RawGeneratorToken<Y, A>>) -> *mut GeneratorArg<Y, A> {
        // SAFETY: id was guanteed to be valid when constructing this waker
        if unsafe { *self.id } != token.id() {
            panic!("waker id does not match generator id");
        }

        self.arg as *mut _
    }
}

unsafe fn waker_clone(ptr: *const ()) -> RawWaker {
    let waker = &*(ptr as *const GeneratorWaker);
    waker.clone_waker()
}

unsafe fn waker_wake_by_ref(ptr: *const ()) {
    let waker = &*(ptr as *const GeneratorWaker);
    waker.wake_by_ref()
}

static GENERATOR_WAKER_VTABLE: RawWakerVTable =
    RawWakerVTable::new(waker_clone, waker_wake_by_ref, waker_wake_by_ref, drop);

static NOOP_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(noop_clone, drop, drop, drop);

pub(crate) unsafe fn noop_clone(_: *const ()) -> RawWaker {
    noop_waker()
}

pub(crate) fn noop_waker() -> RawWaker {
    RawWaker::new(std::ptr::null(), &NOOP_WAKER_VTABLE)
}
