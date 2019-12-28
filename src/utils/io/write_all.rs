use core::pin::Pin;

use futures_core::{future::Future, task::Poll};
use futures_util::{future::poll_fn, ready};

use super::Write;

#[derive(Debug)]
pub enum Error<T> {
    WriteZero,
    Other(T),
}

impl<T> From<T> for Error<T> {
    fn from(err: T) -> Self {
        Error::Other(err)
    }
}

#[macro_export]
macro_rules! readyd {
    ($position:expr, $e:expr $(,)?) => {
        match $e {
            Poll::Ready(t) => t,
            Poll::Pending => {
                //hprintln!("n{:?}", $position);
                return Poll::Pending;
            }
        }
    };
}

pub fn write_all<'a, W: Write + 'a>(
    mut this: Pin<&'a mut W>,
    buf: impl AsRef<[u8]> + 'a,
) -> impl Future<Output = Result<(), Error<W::Error>>> + 'a {
    let mut position = 0;
    poll_fn(move |cx| {
        let buf = buf.as_ref();
        while position < buf.len() {
            let amount = readyd!(position, this.as_mut().poll_write(cx, &buf[position..]))?;
            position += amount;
            if amount == 0 {
                Err(Error::WriteZero)?;
            }
        }
        Poll::Ready(Ok(()))
    })
}
