use std::marker::PhantomData;

use futures::{TryStreamExt, Future, future};
use tokio::{io::{AsyncWrite, AsyncRead}, spawn};
use tokio_util::compat::{TokioAsyncReadCompatExt, FuturesAsyncReadCompatExt, Compat};
use yamux::{Control, Connection, Config, Mode, ConnectionError, WindowUpdateMode};

use crate::ProstClientStream;



pub struct YamuxCtrl<S> {
    ctrl: Control,
    _s: PhantomData<S>,
}

impl<S> YamuxCtrl<S> 
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static
{
    pub fn new_client(stream: S, config: Option<Config>) -> Self {
        Self::new(stream, config, Mode::Client, |_stream| future::ready(Ok(())))
    }

    pub fn new_server<F, Fut>(stream: S, config: Option<Config>, f: F) -> Self 
    where
        F: FnMut(yamux::Stream) -> Fut,
        F: Send + 'static,
        Fut: Future<Output = Result<(), ConnectionError>> + Send + 'static
    {
        Self::new(stream, config, Mode::Server, f)
    }

    fn new<F, Fut>(stream: S, config: Option<Config>, mode: Mode, f: F) -> Self 
    where
        F: FnMut(yamux::Stream) -> Fut,
        F: Send + 'static,
        Fut: Future<Output = Result<(), ConnectionError>> + Send + 'static,
    {

        let mut config = config.unwrap_or_default();
        config.set_window_update_mode(WindowUpdateMode::OnRead);

        let conn = Connection::new(stream.compat(), config, mode);

        // 0.11 failed
        // let (ctrl, conn) = Control::new(conn);
        // spawn(conn.try_for_each_concurrent(None, f));

        let ctrl = conn.control();
        spawn(yamux::into_stream(conn).try_for_each_concurrent(None, f));

        Self {
            ctrl,
            _s: PhantomData,
        }
    }

    pub async fn open_stream(&mut self) -> crate::Result<ProstClientStream<Compat<yamux::Stream>>> {
        let stream = self.ctrl.open_stream().await?;
        Ok(ProstClientStream::new(stream.compat()))
    }
}


#[cfg(test)]
mod tests {
    use std::{task::{Poll, Context}, pin::Pin};

    use bytes::{BytesMut, BufMut};
    use tokio::io::{AsyncRead, ReadBuf, AsyncWrite};

    use crate::network::multiplex::YamuxCtrl;


    #[derive(Debug)]
    struct DummyStream {
        stream: BytesMut
    }

    impl AsyncRead for DummyStream {
        fn poll_read(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<std::io::Result<()>> {
            let this = self.get_mut();
            let len = std::cmp::min(buf.capacity(), this.stream.len());

            let data = this.stream.split_to(len);
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
            self.get_mut().stream.put_slice(buf);
            Poll::Ready(Ok(buf.len()))
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
            Poll::Ready(Ok(()))
        }

        fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
            Poll::Ready(Ok(()))
        }
    }

    #[tokio::test]
    async fn open_stream_should_work() {
        let stream = DummyStream { stream: BytesMut::new() };
        let mut ctrl = YamuxCtrl::new_client(stream, None);
        let res = ctrl.open_stream().await;
        assert!(res.is_ok());
    }
}