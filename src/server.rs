use std::io::Write;

use tokio::net::{TcpListener, TcpStream};

async fn handle_connection(socket: TcpStream) -> anyhow::Result<()> {
    loop {
        socket.readable().await?;

        let mut buf = [0; 4096];

        match socket.try_read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                println!("read {} bytes", n);
                std::io::stdout().write_all(&buf[..n])?;
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }

    Ok(())
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
