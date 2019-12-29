use core::{
    cmp,
    fmt::Debug,
    mem,
    pin::Pin,
    task::{self, Poll},
};

use crate::hal::serial::IntrStream;
use embedded_hal::serial::Write as SerialWrite;

pub trait Write {
    type Error: Debug;

    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::Error>>;

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> Poll<Result<(), Self::Error>>;

    fn poll_close(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> Poll<Result<(), Self::Error>>;
}

//impl<W> Write for Pin<&mut W>
//where
//    W: Write,
//{
//    type Error = <W as Write>::Error;
//
//    fn poll_write(
//        self: Pin<&mut Self>,
//        cx: &mut task::Context<'_>,
//        buf: &[u8],
//    ) -> Poll<Result<usize, Self::Error>> {
//        <W as Write>::poll_write(Pin::get_mut(self).as_mut(), cx, buf)
//    }
//
//    fn poll_flush(
//        self: Pin<&mut Self>,
//        cx: &mut task::Context<'_>,
//    ) -> Poll<Result<(), Self::Error>> {
//        <W as Write>::poll_flush(Pin::get_mut(self).as_mut(), cx)
//    }
//
//    fn poll_close(
//        self: Pin<&mut Self>,
//        cx: &mut task::Context<'_>,
//    ) -> Poll<Result<(), Self::Error>> {
//        <W as Write>::poll_close(Pin::get_mut(self).as_mut(), cx)
//    }
//}
//
//impl Write for &mut [u8] {
//    type Error = !;
//
//    fn poll_write(
//        mut self: Pin<&mut Self>,
//        _cx: &mut task::Context<'_>,
//        buf: &[u8],
//    ) -> Poll<Result<usize, Self::Error>> {
//        let len = cmp::min(self.len(), buf.len());
//        let (head, tail) = mem::replace(&mut *self, &mut []).split_at_mut(len);
//        head.copy_from_slice(&buf[..len]);
//        *self = tail;
//        Poll::Ready(Ok(len))
//    }
//
//    fn poll_flush(
//        self: Pin<&mut Self>,
//        _cx: &mut task::Context<'_>,
//    ) -> Poll<Result<(), Self::Error>> {
//        Poll::Ready(Ok(()))
//    }
//
//    fn poll_close(
//        self: Pin<&mut Self>,
//        _cx: &mut task::Context<'_>,
//    ) -> Poll<Result<(), Self::Error>> {
//        Poll::Ready(Ok(()))
//    }
//}

impl<W> Write for W
where
    W: SerialWrite<u8>,
    W::Error: Debug,
    W: IntrStream,
{
    type Error = <W as SerialWrite<u8>>::Error;

    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::Error>> {
        let writer = unsafe { self.get_unchecked_mut() };
        let mut pos = 0;
        //hprintln!("b{:?}", buf.len());
        writer.unlisten();

        while (pos < buf.len()) {
            match writer.write(buf[pos]) {
                Poll::Ready(Ok(_)) => {
                    pos += 1;
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending if pos > 0 => {
                    //hprintln!("w {:?}", pos);
                    return Poll::Ready(Ok(pos));
                }
                Poll::Pending => {
                    writer.listen(cx.waker().clone());
                    return Poll::Pending;
                }
            }
        }

        Poll::Ready(Ok(pos))
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        let writer = unsafe { self.get_unchecked_mut() };
        match writer.flush() {
            Poll::Pending => {
                writer.listen(cx.waker().clone());
                Poll::Pending
            }
            Poll::Ready(out) => Poll::Ready(out),
        }
    }

    fn poll_close(
        self: Pin<&mut Self>,
        _cx: &mut task::Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}
