use bytes::{BytesMut, BufMut};
use futures::StreamExt;
use kvserver::CommandRequest;
use prost::Message;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

#[derive(Debug)]
pub struct DummyStream {
    buf: BytesMut,
}

impl AsyncRead for DummyStream {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let size = buf.capacity();
        let data = self.get_mut().buf.split_to(size);
        buf.put_slice(&data);
        std::task::Poll::Ready(Ok(()))
    }
}

impl AsyncWrite for DummyStream {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        let len = buf.len();
        self.get_mut().buf.put_slice(buf);
        std::task::Poll::Ready(Ok(len))
    }

    fn poll_flush(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), std::io::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), std::io::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut buf = BytesMut::new();
    let cmd = CommandRequest::new_hset("t1", "k1", vec![0; 10000].into());
    cmd.encode(&mut buf)?;

    let stream = DummyStream { buf};
    let mut s = Framed::new(stream, LengthDelimitedCodec::new());
    let a = s.next().await;
    println!("a = {:?}", a);

    Ok(())
}