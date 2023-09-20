
use bytes::BytesMut;
use futures::SinkExt;
use kvserver::{ServiceInner, MemoryDb, CommandRequest};
use prost::Message;
use tokio::net::TcpListener;
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LengthDelimitedCodec};




#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let service = ServiceInner::new(MemoryDb::new()).service();
    let listener = TcpListener::bind("127.0.0.1:9900").await?;

    loop {
        let (stream, _addr) = listener.accept().await?;
        let svc = service.clone();
        tokio::spawn(async move {
            // tokio-util 中的 LengthDelimitedCodec 只能处理 FrameCoder 中的未压缩的 frame，无法处理压缩的frame
            let mut framed = Framed::new(stream, LengthDelimitedCodec::new());
            while let Some(Ok(buf))= framed.next().await {
                println!("%%%%%%%%%%%%%%%%%");
                let cmd = CommandRequest::decode(buf).unwrap();
                let mut a = svc.execute(cmd);
                let res = a.next().await.unwrap();
                println!("{:?}", res);
                let mut b = BytesMut::new();
                res.encode(&mut b).unwrap();
                framed.send(b.freeze()).await.unwrap();
            }
        });
    }
}