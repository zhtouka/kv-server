use kvserver::{Result, start_server_with_config};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    start_server_with_config().await
}