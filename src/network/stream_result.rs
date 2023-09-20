use std::{pin::Pin, ops::{Deref, DerefMut}};

use futures::{Stream, StreamExt};
use tokio_stream::once;

use crate::{CommandResponse, Result, KvError};



pub struct StreamResult {
    pub id: i64,
    inner: Pin<Box<dyn Stream<Item = Result<CommandResponse>> + Send>>
}

impl StreamResult {
    pub async fn new<S>(mut stream: S) -> Result<Self> 
    where
        S: Stream<Item = Result<CommandResponse>> + Send + 'static + Unpin
    {
        let id = match stream.next().await {
            Some(Ok(CommandResponse { state_code: 200, values, exit, msg, pairs  })) => {
                if exit {
                    return Ok(Self {
                        id: 0,
                        inner: Box::pin(once(Ok(CommandResponse { state_code: 200, msg, values, pairs, exit }))),
                    });
                }
                if values.is_empty() {
                    return Err(KvError::Invalid("invalid command".into()));
                }
                let id: i64 = (&values[0]).try_into()?;

                id
            },
            _ => return Err(KvError::Invalid("invalid command".into())),
        };
        
        Ok(Self {
            id,
            inner: Box::pin(stream),
        })
    }
}

impl Deref for StreamResult {
    type Target = Pin<Box<dyn Stream<Item = Result<CommandResponse>> + Send>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for StreamResult {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}