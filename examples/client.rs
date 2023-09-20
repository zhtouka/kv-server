use kvserver::{ProstClientStream, CommandRequest};
use tokio::net::TcpStream;



#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:9900").await?;
    let mut s: ProstClientStream<TcpStream> = ProstClientStream::new(stream);
    let cmd = CommandRequest::new_hset("t1", "k1", vec![0; 1000].into());
    // 当 server 无法处理数据时，不会返回内容，execute_unary 则无法拉取的数据，会报错
    let a = s.execute_unary(&cmd).await.unwrap();
    println!("{:?}", a);
    Ok(())
}