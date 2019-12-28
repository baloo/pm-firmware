use core::task::{RawWaker, RawWakerVTable, Waker};

//#[cfg(armv7m)]
mod armv7m;
//#[cfg(armv7m)]
pub use self::armv7m::EmbedWaker;

static EMBED_WAKER_RAW_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(
    |data| unsafe { (*(data as *const EmbedWaker)).raw_waker() },
    |data| unsafe { (*(data as *const EmbedWaker)).wake() },
    |data| unsafe { (*(data as *const EmbedWaker)).wake() },
    |_| (/* Noop */),
);

impl EmbedWaker {
    pub(crate) fn waker(&'static self) -> Waker {
        unsafe { Waker::from_raw(self.raw_waker()) }
    }

    pub(crate) fn raw_waker(&'static self) -> RawWaker {
        RawWaker::new(self as *const _ as *const (), &EMBED_WAKER_RAW_WAKER_VTABLE)
    }
}
