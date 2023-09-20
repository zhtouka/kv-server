mod command_service;
mod topic;
mod topic_service;

use std::sync::Arc;

use futures::stream;

use crate::{
    pb::{command_request::RequestData, CommandRequest, CommandResponse},
    service::{command_service::CommandService, topic_service::TopicService},
    storage::{MemoryDb, Storage},
    KvError,
};

use self::{topic::{Topic, Broadcaster}, topic_service::StreamingResponse};

pub struct Service<Store = MemoryDb> {
    inner: Arc<ServiceInner<Store>>,
    broadcaster: Arc<Broadcaster>
}

impl<Store> Clone for Service<Store> {
    fn clone(&self) -> Self {
        Self { 
            inner: Arc::clone(&self.inner), 
            broadcaster: Arc::clone(&self.broadcaster) 
        }
    }
}

impl<Store: Storage> Service<Store> {
    pub fn execute(&self, cmd: CommandRequest) -> StreamingResponse {
        self.inner.on_received.notify(&cmd);
        let mut res = dispatch(cmd.clone(), &self.inner.store);
        self.inner.on_executed.notify(&res);
        // before send
        self.inner.on_before_send.notify_mut(&mut res);
        if res == CommandResponse::default() {
            dispatch_stream(cmd, Arc::clone(&self.broadcaster))
        } else {
            Box::pin(stream::once(async { Arc::new(res) }))
        }
        
    }
}

fn dispatch(cmd: CommandRequest, store: &impl Storage) -> CommandResponse {
    let data = cmd.request_data;
    let Some(data) = data else {
        return KvError::InvalidCommand("".to_string()).into();
    };
    match data {
        RequestData::Hget(x) => x.execute(store),
        RequestData::Hmget(x) => x.execute(store),
        RequestData::Hset(x) => x.execute(store),
        RequestData::Hmset(x) => x.execute(store),
        RequestData::Hexists(x) => x.execute(store),
        RequestData::Hmexists(x) => x.execute(store),
        RequestData::Hdelete(x) => x.execute(store),
        RequestData::Hmdelete(x) => x.execute(store),
        RequestData::Hgetall(x) => x.execute(store),
        _ => CommandResponse::default(),
    }
}

fn dispatch_stream(cmd: CommandRequest, topic: impl Topic) -> StreamingResponse {

    match cmd.request_data {
        Some(RequestData::Subscribe(x)) => x.execute(topic),
        Some(RequestData::Unsubscribe(x)) => x.execute(topic),
        Some(RequestData::Publish(x)) => x.execute(topic),
        _ => unreachable!(),
    }
}

pub struct ServiceInner<Store> {
    store: Store,
    on_received: Vec<fn(&CommandRequest)>,
    on_executed: Vec<fn(&CommandResponse)>,
    on_before_send: Vec<fn(&mut CommandResponse)>,
    on_after_send: Vec<fn()>,
}

impl<Store: Storage> ServiceInner<Store> {
    pub fn new(store: Store) -> Self {
        Self {
            store,
            on_received: vec![],
            on_executed: vec![],
            on_before_send: vec![],
            on_after_send: vec![],
        }
    }

    pub fn service(self) -> Service<Store> {
        self.into()
    }

    pub fn fn_received(mut self, f: fn(&CommandRequest)) -> Self {
        self.on_received.push(f);
        self
    }

    pub fn fn_executed(mut self, f: fn(&CommandResponse)) -> Self {
        self.on_executed.push(f);
        self
    }

    pub fn fn_before_send(mut self, f: fn(&mut CommandResponse)) -> Self {
        self.on_before_send.push(f);
        self
    }

    pub fn fn_after_send(mut self, f: fn()) -> Self {
        self.on_after_send.push(f);
        self
    }
}

impl<Store: Storage> From<ServiceInner<Store>> for Service<Store> {
    fn from(value: ServiceInner<Store>) -> Self {
        Self {
            inner: Arc::new(value),
            broadcaster: Default::default(),
        }
    }
}

pub trait Notify<Arg> {
    fn notify(&self, args: &Arg);
}

pub trait NotifyMut<Arg> {
    fn notify_mut(&self, args: &mut Arg);
}

impl<Arg> Notify<Arg> for Vec<fn(&Arg)> {
    fn notify(&self, args: &Arg) {
        for f in self {
            f(args)
        }
    }
}

impl<Arg> NotifyMut<Arg> for Vec<fn(&mut Arg)> {
    fn notify_mut(&self, args: &mut Arg) {
        for f in self {
            f(args)
        }
    }
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;

    use crate::{
        pb::{CommandRequest, CommandResponse},
        storage::MemoryDb,
    };

    use super::ServiceInner;

    fn fn_received(cmd: &CommandRequest) {
        println!("on received command request: {:?}", cmd);
    }

    fn fn_executed(cmd: &CommandResponse) {
        println!("on execute command response: {:?}", cmd);
    }

    fn fn_before_send(cmd: &mut CommandResponse) {
        cmd.msg = "OK".to_string();
        println!("on before send command response: {:?}", cmd);
    }

    fn fn_after_send() {
        println!("on after send command response");
    }

    #[tokio::test]
    async fn notify_should_work() {
        let db = MemoryDb::new();
        let service = ServiceInner::new(db)
            .fn_received(fn_received)
            .fn_executed(fn_executed)
            .fn_before_send(fn_before_send)
            .fn_after_send(fn_after_send)
            .service();

        let cmd = CommandRequest::new_hget("t1", "k1");
        let mut stream = service.execute(cmd);
        let cmd = stream.next().await.unwrap();
        assert_eq!(&cmd.msg, "OK");
    }
}
