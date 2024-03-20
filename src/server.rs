use anyhow::bail;
use redis::{deserializer::from_bytes, serializer::to_bytes, value::Value};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};

async fn handle_connection(mut socket: TcpStream) -> anyhow::Result<()> {
    loop {
        socket.readable().await?;

        let mut buf = [0; 4096];

        match socket.try_read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                println!("read {} bytes", n);
                let v: Value = from_bytes(&buf[..n])?;
                println!("{v:?}");
                let Value::Array(Some(x)) = v else {
                    bail!("no");
                };
                let Value::String(Some(command)) = &x[0] else {
                    bail!("no 2")
                };

                println!("got command {command}");
                let ser = to_bytes(&Value::str("PONG"))?;
                socket.write_all(&ser).await?;
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
