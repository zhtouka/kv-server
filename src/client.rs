use std::time::Duration;

use futures::StreamExt;
use kvserver::{Result, CommandRequest, ProstClientStream, KvError, start_client};
use tokio_util::compat::{Compat};

#[tokio::main]
async fn main() -> Result<()> {

    let mut ctrl = start_client().await?;

    let mut stream = ctrl.open_stream().await?;

    let cmd = CommandRequest::new_hset("t1", "k1", "v1".into());
    let res = stream.execute_unary(&cmd).await?;

    println!("res = {:?}", res);

    let stream = ctrl.open_stream().await?;

    let cmd = CommandRequest::subscribe("t1");
    let mut res = stream.execute_streaming(&cmd).await?;
    let id = res.id;

    let stream = ctrl.open_stream().await?;
    start_publish(stream, "t1");
    if let Some(Ok(data)) = res.next().await {
        println!("{:?}", data);
    }
    
    let stream = ctrl.open_stream().await?;
    start_unsubscribe(stream, "t1", id as _);

    let stream = ctrl.open_stream().await?;
    start_publish(stream, "t1");
    if let Some(Ok(data)) = res.next().await {
        assert!(data.exit);
        println!("{:?}", data);
    }
    Ok(())
}

fn start_publish(mut stream: ProstClientStream<Compat<yamux::Stream>>, topic: &str) {
    let cmd = CommandRequest::publish(topic, vec![0.into(), 1.into()]);
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(1)).await;
        stream.execute_unary(&cmd).await?;
        Ok::<_, KvError>(())
    });
}

fn start_unsubscribe(mut stream: ProstClientStream<Compat<yamux::Stream>>, topic: &str, id: u32) {
    let cmd = CommandRequest::unsubscribe(topic, id);
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(1)).await;
        stream.execute_unary(&cmd).await?;
        Ok::<_, KvError>(())
    });
}