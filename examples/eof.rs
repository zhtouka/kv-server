use tokio::io::{self, AsyncReadExt};

use std::io::Cursor;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut reader = Cursor::new(vec![0x00, 0x01, 0x0b]);

    println!("{:?}", reader.read_u32().await);
    Ok(())
}