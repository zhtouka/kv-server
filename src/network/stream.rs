use std::{marker::PhantomData, pin::Pin, task::{Context, Poll, ready}};

use bytes::BytesMut;
use futures::{Stream, FutureExt, Sink};
use tokio::io::{AsyncWrite, AsyncRead};

use crate::{network::frame::read_frame, KvError};

use super::frame::FrameCoder;



pub struct ProstStream<S, In, Out> {
    stream: S,

    rbuf: BytesMut,
    wbuf: BytesMut,

    written: usize,

    _in: PhantomData<In>,
    _out: PhantomData<Out>,
}

impl<S, In, Out> ProstStream<S, In, Out> {
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            rbuf: BytesMut::new(),
            wbuf: BytesMut::new(),
            written: 0,
            _in: PhantomData,
            _out: PhantomData,
        }
    }
}

impl<S, In, Out> Stream for ProstStream<S, In, Out> 
where
    S: AsyncRead + AsyncWrite + Send + Unpin,
    In: Send + Unpin + FrameCoder,
    Out: Send + Unpin,
{
    type Item = crate::Result<In>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        let mut buf = this.rbuf.split_off(0);
        let fut = read_frame(&mut this.stream, &mut buf);
        ready!(Box::pin(fut).poll_unpin(cx))?;

        let item = In::decode_frame(&mut buf);
        this.rbuf.unsplit(buf);
        Poll::Ready(Some(item))
    }
}

impl<S, In, Out> Sink<&Out> for ProstStream<S, In, Out> 
where
    S: AsyncRead + AsyncWrite + Send + Unpin,
    In: Send + Unpin,
    Out: Send + Unpin + FrameCoder
{
    type Error = KvError;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, item: &Out) -> Result<(), Self::Error> {
        let this = self.get_mut();

        item.encode_frame(&mut this.wbuf)?;

        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.get_mut();

        while this.written != this.wbuf.len() {
            let n = ready!(Pin::new(&mut this.stream).poll_write(cx, &this.wbuf))?;
            this.written += n;
        }
        this.written = 0;
        this.wbuf.clear();

        ready!(Pin::new(&mut this.stream).poll_flush(cx))?;

        Poll::Ready(Ok(()))
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        ready!(self.as_mut().poll_flush(cx))?;

        ready!(Pin::new(&mut self.stream).poll_shutdown(cx))?;

        Poll::Ready(Ok(()))
    }
}

impl<S, In, Out> Unpin for ProstStream<S, In, Out> {}


#[cfg(test)]
mod tests {
    use std::{pin::Pin, task::{Context, Poll}};

    use bytes::{BytesMut, BufMut};
    use futures::{SinkExt, StreamExt};
    use tokio::io::{AsyncRead, ReadBuf, AsyncWrite};

    use crate::pb::CommandRequest;

    use super::ProstStream;



    struct DummyStream {
        stream: BytesMut
    }

    impl AsyncRead for DummyStream {
        fn poll_read(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<std::io::Result<()>> {
            let len = buf.capacity();

            let size = self.stream.capacity();
            assert!(size > 0);

            let data = self.get_mut().stream.split_to(len);
            buf.put_slice(&data);

            Poll::Ready(Ok(()))
        }
    }

    impl AsyncWrite for DummyStream {
        fn poll_write(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<Result<usize, std::io::Error>> {
            let len = buf.len();
            self.get_mut().stream.put_slice(buf);
            Poll::Ready(Ok(len))
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
            Poll::Ready(Ok(()))
        }

        fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
            Poll::Ready(Ok(()))
        }
    }

    #[tokio::test]
    async fn prost_stream_should_work() {
        let stream = DummyStream { stream: BytesMut::new() };
        let mut stream = ProstStream::<_, CommandRequest, CommandRequest>::new(stream);
        let cmd = CommandRequest::new_hget("t1", "k1");
        let res = stream.send(&cmd).await;
        assert!(res.is_ok());
        let res = stream.send(&cmd).await;
        assert!(res.is_ok());

        if let Some(Ok(res)) = stream.next().await {
            assert_eq!(res, cmd);
        }
        if let Some(Ok(res)) = stream.next().await {
            assert_eq!(res, cmd);
        }
    }
}