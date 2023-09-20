mod commandline;
mod config;
mod error;
mod network;
mod pb;
mod service;
mod storage;

pub use commandline::{get_command, CommandType};
use storage::{SledDb, Storage};
pub use crate::config::*;
pub use error::*;
pub use network::{ProstClientStream, ProstServerStream, YamuxCtrl};
pub use pb::*;
pub use service::ServiceInner;
pub use storage::MemoryDb;
use tokio::{net::{TcpListener, TcpStream}};
use tokio_util::compat::FuturesAsyncReadCompatExt;
use tracing::log::info;

lazy_static::lazy_static! {
    pub static ref CONFIG: ServerSettings = config().server;
}

pub async fn start_server_with_config() -> Result<()> {
    let addr = format!("127.0.0.1:{}", CONFIG.port);
    let name = CONFIG.store.name.as_ref();
    let path = CONFIG.store.path.as_deref();
    match (name, path) {
        ("sleddb", Some(path)) => start_server(&addr, SledDb::new(path)).await,
        _ => start_server(&addr, MemoryDb::new()).await
    }
}

pub async fn start_server<Store: Storage>(addr: &str, store: Store) -> Result<()> {

    let listener = TcpListener::bind(&addr).await?;
    info!("Listenning address: {:?}", &addr);
    let service = ServiceInner::new(store).service();

    loop {
        let (stream, _) = listener.accept().await?;
        let svc = service.clone();
        tokio::spawn(async move {
            YamuxCtrl::new_server(stream, None, move |stream| {
                let mut stream = ProstServerStream::new(stream.compat(), svc.clone());
                async move {
                    stream.process().await.expect("failed to server stream execute");
                    Ok(())
                }
            });
        });
    }
}



pub async fn start_server_with_yamux(addr: &str) -> Result<()> {

    let listener = TcpListener::bind(addr).await?;
    info!("Listenning address: {:?}", addr);
    let service = ServiceInner::new(MemoryDb::new()).service();

    loop {
        let (stream, _) = listener.accept().await?;
        let svc = service.clone();
        YamuxCtrl::new_server(stream, None, move |stream| {
            let mut stream = ProstServerStream::new(stream.compat(), svc.clone());
            async move {
                stream.process().await.expect("failed to server stream execute");
                Ok(())
            }
        });
    }
}


pub async fn start_client() -> Result<YamuxCtrl<TcpStream>> {

    let addr = format!("127.0.0.1:{}", CONFIG.port);

    let stream = TcpStream::connect(&addr).await?;

    Ok(YamuxCtrl::new_client(stream, None))
}