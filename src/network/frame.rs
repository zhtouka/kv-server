use std::io::{Write, Read};

use bytes::{BytesMut, BufMut, Buf};
use flate2::{write::GzEncoder, Compression, read::GzDecoder};
use prost::Message;
use tokio::io::{AsyncRead, AsyncReadExt};

use crate::{Result, KvError, pb::{CommandRequest, CommandResponse}};


const LEN_LEN: usize = 4;
const MAX_FRAME: usize = 2 * 1024 * 1024 - 1;
const COMPRESSION_LIMIT: usize = 1436;
const COMPRESSION_BIT: usize = 1 << 31;

pub trait FrameCoder 
where
    Self: Sized + Message + Default,
{
    fn encode_frame(&self, buf: &mut BytesMut) -> Result<()> {
        let size = self.encoded_len();
        if size > MAX_FRAME {
            return Err(KvError::FrameError);
        }

        buf.put_u32(size as _);
        if size > COMPRESSION_LIMIT {
            let mut buf1 = Vec::with_capacity(size);
            self.encode(&mut buf1)?;

            let payload = buf.split_off(LEN_LEN);
            buf.clear();

            let mut encoder = GzEncoder::new(payload.writer(), Compression::default());
            encoder.write_all(&buf1[..])?;
            let payload = encoder.finish()?.into_inner();

            buf.put_u32((payload.len() | COMPRESSION_BIT) as _);
            buf.unsplit(payload);
        } else {
            self.encode(buf)?;
        }

        Ok(())
    }

    fn decode_frame(buf: &mut BytesMut) -> Result<Self> {
        let header = buf.get_u32();
        let (len, compression) = decode_header(header as _);

        if compression {
            let mut buf1 = Vec::with_capacity(len * 2);
            let mut decoder = GzDecoder::new(&buf[..len]);
            decoder.read_to_end(&mut buf1)?;

            buf.advance(len);
            
            Ok(Self::decode(&buf1[..buf1.len()])?)
        } else {
            let res = Self::decode(&buf[..len])?;
            buf.advance(len);
            Ok(res)
        }
    }
}

fn decode_header(header: usize) -> (usize, bool) {
    let is_compression = header | COMPRESSION_BIT == header;
    let len = header & (COMPRESSION_BIT - 1);
    (len, is_compression)
}

impl FrameCoder for CommandRequest {}
impl FrameCoder for CommandResponse {}

pub async fn read_frame<S>(stream: &mut S, buf: &mut BytesMut) -> Result<()> 
where
    S: AsyncRead + Unpin + Send
{
    // 上游会使用 while let Ok(x) = s.next() 持续拉取，当 stream 中没有内容时返回错误，终止循环
    let header = stream.read_u32().await.map_err(|_| KvError::FrameError)?;
    let (len, _compression) = decode_header(header as _);

    buf.reserve(len + LEN_LEN);
    buf.put_u32(header);

    unsafe { buf.advance_mut(len) }
    stream.read_exact(&mut buf[LEN_LEN..]).await?;
    Ok(())
}


#[cfg(test)]
mod tests {
    use std::{pin::Pin, task::{Context, Poll}};

    use bytes::{BytesMut};
    use prost::Message;
    use tokio::io::{AsyncRead, ReadBuf};

    use crate::{pb::{CommandRequest, CommandResponse}, Value};

    use super::{FrameCoder, read_frame};


    #[test]
    fn command_frame_coder_should_work() {
        let cmd = CommandRequest::new_hget("t1", "k1");
        let mut buf = BytesMut::new();
        let res = cmd.encode_frame(&mut buf);
        assert!(res.is_ok());
        let res = CommandRequest::decode_frame(&mut buf);
        assert!(res.is_ok());
        println!("{:?}", res);
        assert_eq!(res.unwrap(), cmd);

        let cmd = CommandResponse::ok();
        let res = cmd.encode_frame(&mut buf);
        assert!(res.is_ok());
        let res = CommandResponse::decode_frame(&mut buf);
        assert!(res.is_ok());
        println!("{:?}", res);
        assert_eq!(res.unwrap(), cmd);
    }

    #[test]
    fn command_compression_frame_coder_should_work() {
        let mut buf = BytesMut::new();
        let mut cmd = CommandResponse::ok();
        cmd.values = vec![0.into(); 10000];
        assert!(cmd.encoded_len() > 1436);

        let res = cmd.encode_frame(&mut buf);
        assert!(res.is_ok());
        
        let res = CommandResponse::decode_frame(&mut buf);

        assert!(res.is_ok());
        assert_eq!(res.unwrap(), cmd);
    }

    #[derive(Debug)]
    struct DummyStream {
        stream: BytesMut,
    }

    impl AsyncRead for DummyStream {
        fn poll_read(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<std::io::Result<()>> {
            let len = buf.capacity();

            let data = self.get_mut().stream.split_to(len);
            buf.put_slice(&data);
            
            Poll::Ready(Ok(()))
        }
    }

    #[tokio::test]
    async fn read_frame_should_work() {
        let cmd = CommandRequest::new_hset("t1", "k1", vec![0; 10000].into());
        let mut buf = BytesMut::new();
        let res = cmd.encode_frame(&mut buf);

        assert!(res.is_ok());
        let mut stream = DummyStream { stream: buf };
        let mut buf = BytesMut::new();
        let res = read_frame(&mut stream, &mut buf).await;
        assert!(res.is_ok());

        let res = CommandRequest::decode_frame(&mut buf);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), cmd);

        let cmd: CommandResponse = Value { value: None}.into();
        let mut buf = BytesMut::new();
        let res = cmd.encode_frame(&mut buf);

        assert!(res.is_ok());
        let mut stream = DummyStream { stream: buf };
        let mut buf = BytesMut::new();
        let res = read_frame(&mut stream, &mut buf).await;
        assert!(res.is_ok());

        let res = CommandResponse::decode_frame(&mut buf);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), cmd);
    }
}