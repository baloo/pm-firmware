use core::sync::atomic::{AtomicBool, Ordering};

use cortex_m_semihosting::hprintln;

pub struct EmbedWaker {
    woken: AtomicBool,
}

impl EmbedWaker {
    pub(crate) const fn new() -> Self {
        EmbedWaker {
            woken: AtomicBool::new(false),
        }
    }

    pub(crate) fn wake(&self) {
        //hprintln!("6");
        self.woken.store(true, Ordering::Release);
        // we send an event in case this was a non-interrupt driven wake
        cortex_m::asm::sev();
    }

    pub(crate) fn test_and_clear(&self) -> bool {
        //hprintln!("1 {:?}", self.woken);
        self.woken.swap(false, Ordering::AcqRel)
    }

    pub(crate) fn sleep() {
        cortex_m::asm::wfe();
    }
}
