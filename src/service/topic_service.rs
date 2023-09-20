use std::{sync::Arc, pin::Pin};

use futures::Stream;
use tokio_stream::{wrappers::UnboundedReceiverStream, once};

use crate::{Subscribe, CommandResponse, Unsubscribe, Publish};

use super::topic::{Topic};


pub type StreamingResponse = Pin<Box<dyn Stream<Item = Arc<CommandResponse>> + Send>>;


pub trait TopicService {
    fn execute(self, topic: impl Topic) -> StreamingResponse;
}

impl TopicService for Subscribe {
    fn execute(self, topic: impl Topic) -> StreamingResponse {
        let rx = topic.subscribe(&self.topic);
        Box::pin(UnboundedReceiverStream::new(rx))
    }
}

impl TopicService for Unsubscribe {
    fn execute(self, topic: impl Topic) -> StreamingResponse {
        topic.unsubscribe(&self.topic, self.id);
        Box::pin(once(Arc::new(CommandResponse::exit())))
    }
}

impl TopicService for Publish {
    fn execute(self, topic: impl Topic) -> StreamingResponse {
        topic.publish(self.topic, Arc::new(self.data.into()));
        Box::pin(once(Arc::new(CommandResponse::exit())))
    }
}