// MIT Licensed and apache
//
// This is a ripoff from https://github.com/Nemo157/embrio-rs/tree/master/embrio-executor

use core::{
    future::Future,
    task::{self, Poll},
};

use cortex_m::interrupt::free;
use pin_utils::pin_mut;

mod waker;
use self::waker::EmbedWaker;

/// A `no_std` compatible, allocation-less, single-threaded futures executor;
/// targeted at supporting embedded use-cases.
///
/// See the [crate docs](crate) for more details.
pub struct Executor {
    waker: EmbedWaker,
}

impl Executor {
    /// Create a new instance of [`Executor`].
    ///
    /// See the [crate docs](crate) for more details.
    pub const fn new() -> Executor {
        Executor {
            waker: EmbedWaker::new(),
        }
    }

    /// Block on a specific [`Future`] until it completes, returning its output
    /// when it does.
    ///
    /// See the [crate docs](crate) for more details.
    pub fn block_on<F: Future>(&'static mut self, future: F) -> F::Output {
        pin_mut!(future);

        let waker = self.waker.waker();
        let mut context = task::Context::from_waker(&waker);

        loop {
            let out = free(|_| future.as_mut().poll(&mut context));

            if let Poll::Ready(val) = out {
                return val;
            } else {
                while !self.waker.test_and_clear() {
                    EmbedWaker::sleep()
                }
            }
        }
    }
}
