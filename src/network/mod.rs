use futures::{StreamExt, SinkExt};
use tokio::io::{AsyncWrite, AsyncRead};
use tracing::log::warn;

use crate::{pb::{CommandResponse, CommandRequest}, service::Service, KvError, storage::Storage};
use crate::Result;

use self::stream::ProstStream;

mod frame;
mod multiplex;
mod stream;
mod stream_result;

pub use multiplex::YamuxCtrl;
pub use stream_result::StreamResult;

pub struct ProstServerStream<S, Store> {
    stream: ProstStream<S, CommandRequest, CommandResponse>,
    service: Service<Store>
}

impl<S, Store> ProstServerStream<S, Store> 
where
    S: AsyncRead + AsyncWrite + Send + Unpin,
    Store: Storage
{
    pub fn new(stream: S, service: Service<Store>) -> Self {
        let stream = ProstStream::new(stream);

        Self {
            stream,
            service,
        }
    }

    pub async fn process(&mut self) -> Result<()> {
        while let Some(Ok(cmd)) = self.stream.next().await {
            let mut stream = self.service.execute(cmd);
            while let Some(cmd) = stream.next().await {
                // println!("{:?}", cmd);
                if let Err(_) = self.stream.send(&cmd).await {
                    warn!("Failed to send command response")
                }
            }
        }
        Ok(())
    }
}

pub struct ProstClientStream<S> {
    stream: ProstStream<S, CommandResponse, CommandRequest>,
}

impl<S> ProstClientStream<S>
where
    S: AsyncRead + AsyncWrite + Send + Unpin + 'static
{
    pub fn new(stream: S) -> Self {
        Self {
            stream: ProstStream::new(stream),
        }
    }

    pub async fn execute_unary(&mut self, cmd: &CommandRequest) -> Result<CommandResponse> {
        self.stream.send(cmd).await?;
        self.stream.next().await.ok_or_else(|| KvError::Internal("Didn't get any response".into()))?
    }

    pub async fn execute_streaming(self, cmd: &CommandRequest) -> Result<StreamResult> {
        let mut this = self.stream;

        this.send(cmd).await?;
        this.close().await?;

        StreamResult::new(this).await
    }
}