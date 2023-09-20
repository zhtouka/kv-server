use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};
use futures::StreamExt;
use kvserver::{start_server_with_yamux, YamuxCtrl, CommandRequest};
use rand::seq::SliceRandom;
use tokio::net::{TcpStream};
use tracing::log::info;

async fn start_server() -> kvserver::Result<()> {
    let addr = "127.0.0.1:9900";
    tokio::spawn(async move {
        start_server_with_yamux(addr).await?;
        Ok::<_, kvserver::KvError>(())
    });
    Ok(())
}

async fn connect() -> kvserver::Result<YamuxCtrl<TcpStream>> {
    let stream = TcpStream::connect("127.0.0.1:9900").await?;
    let ctrl = YamuxCtrl::new_client(stream, None);
    Ok(ctrl)
}

async fn start_subscribe(topic: &str) -> kvserver::Result<()> {
    let mut ctrl = connect().await?;
    let stream = ctrl.open_stream().await?;
    info!("C(subscriber): stream opened");
    let cmd = CommandRequest::subscribe(topic);

    tokio::spawn(async move {
        let mut stream = stream.execute_streaming(&cmd).await?;
        while let Some(Ok(res)) = stream.next().await {
            drop(res);
        }
        Ok::<_, kvserver::KvError>(())
    });

    Ok(())
}

async fn start_publish(topic: &str, values: &[&str]) -> kvserver::Result<()> {
    let mut rng = rand::thread_rng();
    let v = values.choose(&mut rng).unwrap();

    let mut ctrl = connect().await?;
    let stream = ctrl.open_stream().await?;
    info!("C(subscriber): stream opened");
    let cmd = CommandRequest::publish(topic, vec![(*v).into()]);

    let mut stream = stream.execute_streaming(&cmd).await?;
    if let Some(Ok(res)) = stream.next().await {
        drop(res);
    }

    Ok(())
}

fn pubsub(c: &mut Criterion) {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .thread_name("pubsub")
        .enable_all()
        .build()
        .unwrap();
    let values = ["Hello", "Tyr", "123", "world"];
    let topic = "hobby";

    runtime.block_on(async {
        eprintln!("preparing server and subscribers");
        start_server().await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        for _ in 0..100 {
            start_subscribe(topic).await.unwrap();
            eprint!(".");
        }
        eprintln!("Done");
    });

    c.bench_function("publishing", move |b| {
        b.to_async(&runtime)
            .iter(|| async { start_publish(topic, &values).await.unwrap() })
    });
}

criterion_group!{
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = pubsub
}

criterion_main!(benches);