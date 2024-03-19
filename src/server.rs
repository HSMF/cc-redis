use anyhow::bail;
use tokio::net::{TcpListener, TcpStream};

async fn handle_connection(socket: TcpStream) -> anyhow::Result<()> {
    bail!("no!")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:6379").await?;
    dbg!(redis::add(1, 2));
    loop {
        let (socket, _) = listener.accept().await?;
        match handle_connection(socket).await {
            Ok(_) => {}
            Err(e) => eprintln!("Error {e}"),
        }
    }
}
