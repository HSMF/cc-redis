use std::sync::OnceLock;

use clap::Parser;
use redis::{commands::App, deserializer::from_bytes, value::Value};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};

static APP: OnceLock<App> = OnceLock::new();

async fn handle_connection(mut socket: TcpStream) -> anyhow::Result<()> {
    let app = APP.get().unwrap();
    loop {
        socket.readable().await?;

        let mut buf = [0; 4096];

        match socket.try_read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let v: Value = from_bytes(&buf[..n])?;
                // println!("{v:?}");
                let response = app.dispatch_command(v).await;
                // println!("{response:?}");
                // use std::io::Write;
                // std::io::stderr().write_all(&ser)?;
                socket.write_all(&response).await?;
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

#[derive(clap::Parser)]
struct Cli {
    #[clap(long)]
    dir: Option<String>,
    #[clap(long)]
    dbfilename: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut app = App::new();
    if let Some(dir) = cli.dir {
        app.set_config("dir".into(), dir);
    }
    if let Some(dbfilename) = cli.dbfilename {
        app.set_config("dbfilename".into(), dbfilename);
    }

    APP.set(app).unwrap();
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
