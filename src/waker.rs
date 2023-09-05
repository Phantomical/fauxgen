use std::ptr::NonNull;
use std::task::{RawWaker, RawWakerVTable, Waker};

use crate::common::GeneratorArg;
use crate::TokenId;

#[repr(C)]
pub(crate) struct GeneratorWaker {
    waker: Option<NonNull<Waker>>,
    id: *mut TokenId,
    arg: *mut (),
}

impl GeneratorWaker {
    pub unsafe fn new<Y, A>(
        waker: Option<&Waker>,
        arg: &mut GeneratorArg<Y, A>,
        id: &mut TokenId,
    ) -> Self {
        Self {
            waker: waker.map(NonNull::from),
            arg: arg as *mut _ as *mut _,
            id,
        }
    }

    pub unsafe fn to_waker(&self) -> Waker {
        Waker::from_raw(RawWaker::new(
            self as *const _ as *const (),
            &GENERATOR_WAKER_VTABLE,
        ))
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
            None => crate::noop::raw_waker(),
        }
    }

    pub fn set_id(&self, id: TokenId) {
        unsafe { *self.id = id };
    }

    pub fn arg<Y, A>(&self, id: TokenId) -> *mut GeneratorArg<Y, A> {
        if unsafe { *self.id != id } {
            panic!("attempted to yield to the waker for a different generator");
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

pub(crate) static GENERATOR_WAKER_VTABLE: RawWakerVTable =
    RawWakerVTable::new(waker_clone, waker_wake_by_ref, waker_wake_by_ref, drop);
