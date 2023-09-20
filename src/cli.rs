use futures::StreamExt;
use kvserver::{get_command, CommandType, start_client};



#[tokio::main]
async fn main() -> kvserver::Result<()> {
    let mut ctrl = start_client().await?;

    let mut stream = ctrl.open_stream().await?;

    let cmd = get_command();

    match cmd {
        CommandType::Unary(cmd) => {
            let res = stream.execute_unary(&cmd).await?;
            println!("{:?}", res);
        },
        CommandType::Stream(cmd) => {
            let mut stream = stream.execute_streaming(&cmd).await?;
            while let Some(Ok(res)) = stream.next().await {
                println!("{:?}", res);
            }
        },
    };

    Ok(())
}