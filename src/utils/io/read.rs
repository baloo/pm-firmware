use core::{
    cmp,
    fmt::Debug,
    pin::Pin,
    task::{self, Poll},
};

use crate::hal::serial::{Event, Intr, IntrStream, Serial};
use embedded_hal::serial::Read as SerialRead;

pub trait Read {
    type Error: Debug;

    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>>;
}

//impl<R> Read for Pin<&mut R>
//where
//    R: Read,
//{
//    type Error = <R as Read>::Error;
//
//    fn poll_read(
//        self: Pin<&mut Self>,
//        cx: &mut task::Context<'_>,
//        buf: &mut [u8],
//    ) -> Poll<Result<usize, Self::Error>> {
//        <R as Read>::poll_read(Pin::get_mut(self).as_mut(), cx, buf)
//    }
//}
//
//impl Read for &[u8] {
//    type Error = !;
//
//    fn poll_read(
//        mut self: Pin<&mut Self>,
//        _cx: &mut task::Context<'_>,
//        buf: &mut [u8],
//    ) -> Poll<Result<usize, Self::Error>> {
//        let len = cmp::min(self.len(), buf.len());
//        let (head, tail) = self.split_at(len);
//        buf[..len].copy_from_slice(head);
//        *self = tail;
//        Poll::Ready(Ok(len))
//    }
//}

impl<R> Read for R
where
    R: SerialRead<u8>,
    <R as SerialRead<u8>>::Error: Debug,
    R: IntrStream,
{
    type Error = <R as SerialRead<u8>>::Error;

    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>> {
        let mut reader = unsafe { self.get_unchecked_mut() };
        let mut pos = 0;

        reader.unlisten();
        while (pos < buf.len()) {
            let val = reader.read();
            match val {
                Poll::Ready(Ok(val)) => {
                    buf[pos] = val;
                    pos += 1;
                    continue;
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending if pos > 0 => return Poll::Ready(Ok(pos)),
                Poll::Pending => {
                    reader.listen(cx.waker().clone());
                    return Poll::Pending;
                }
            }
        }

        Poll::Ready(Ok(pos))
    }
}
