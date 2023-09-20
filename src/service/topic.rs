use std::sync::{atomic::{AtomicU32, Ordering}, Arc};

use dashmap::{DashMap, DashSet};
use tokio::sync::mpsc::{self, UnboundedSender, UnboundedReceiver};
use tracing::log::warn;

use crate::{CommandResponse};

// 不能使用 const
static NEXT_ID: AtomicU32 = AtomicU32::new(1);

fn get_next_id() -> u32 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}


#[derive(Default, Debug, Clone)]
pub struct Broadcaster {
    topics: DashMap<String, DashSet<u32>>,
    sender: DashMap<u32, UnboundedSender<Arc<CommandResponse>>>
}


pub trait Topic {
    fn subscribe(&self, topic: impl Into<String>) -> UnboundedReceiver<Arc<CommandResponse>>;

    fn unsubscribe(&self, topic: &str, id: u32);

    fn publish(self, topic: String, data: Arc<CommandResponse>);
}

impl Topic for Arc<Broadcaster> {
    fn subscribe(&self, topic: impl Into<String>) -> UnboundedReceiver<Arc<CommandResponse>> {
        let id = get_next_id();
        let set = self.topics.entry(topic.into()).or_default();
        set.insert(id);

        let (tx, rx) = mpsc::unbounded_channel::<Arc<CommandResponse>>();

        let tx1 = tx.clone();
        let cmd = (id as i64).into();
        tokio::spawn(async move {
            if let Err(e) = tx1.send(Arc::new(cmd)) {
                warn!("Failed to send command: {:?}", e);
            }
        });

        self.sender.insert(id, tx);

        rx
    }

    fn unsubscribe(&self, topic: &str, id: u32) {
        if let Some(set) = self.topics.get(topic) {
            set.remove(&id);
            if set.is_empty() {
                drop(set);
                self.topics.remove(topic);
            }
        }

        let (_, sender) = self.sender.remove(&id).clone().unwrap();
        tokio::spawn(async move {
            if let Err(e) = sender.send(Arc::new(CommandResponse::exit())) {
                warn!("Failed to send command: {:?}", e);
            }
        });
    }

    fn publish(self, topic: String, data: Arc<CommandResponse>) {
        tokio::spawn(async move {
            if let Some(set) = self.topics.get(&topic) {
                let set1 = set.clone();
                set1
                    .into_iter()
                    .for_each(|id| {
                        if let Some(sender) = self.sender.get(&id) {
                            if let Err(e) = sender.send(data.clone()) {
                                warn!("Failed to send command: {:?}", e);
                            }
                        }
                    })
            }
        });
    }
}


#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::CommandResponse;

    use super::{Broadcaster, Topic, get_next_id};

    #[test]
    fn get_next_id_should_work() {
        let id = get_next_id();
        assert_eq!(id, 1);
        let id = get_next_id();
        assert_eq!(id, 2);
    }


    #[tokio::test]
    async fn topic_should_work() {
        let bc = Arc::new(Broadcaster::default());

        let mut rx = bc.subscribe("topic");
        
        if let Some(cmd) = rx.recv().await {
            let id = cmd.values[0].clone();
            assert_eq!(id, 1.into());
        }
        let mut rx1 = bc.subscribe("topic");

        let cmd = Arc::new(CommandResponse::ok());
        bc.clone().publish("topic".into(), cmd.clone());

        let res1 = rx.recv().await.unwrap();
        assert_eq!(res1, cmd);
        rx1.recv().await.unwrap();
        let res2 = rx1.recv().await.unwrap();
        assert_eq!(res1, res2);

        bc.unsubscribe("topic", 1);
        bc.publish("topic".into(), cmd.clone());

        let res1 = rx.recv().await.unwrap();
        assert_eq!(res1, Arc::new(CommandResponse::exit()));
        let res2 = rx1.recv().await.unwrap();
        assert_eq!(res2, cmd);
    }

}