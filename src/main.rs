use anyhow::Result;
use futures::SinkExt;
use my_redis::backend::Storage;
use my_redis::codec::Codec;
use tokio::net::{TcpListener, TcpStream};
use tokio_stream::StreamExt;
use tokio_util::codec::Framed;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let addr = "0.0.0.0:6379";
    let storage = Storage::new();
    let listener = TcpListener::bind(addr).await?;

    info!("Listening on: {}", addr);

    loop {
        let (socket, _) = listener.accept().await?;
        let s = storage.clone();
        tokio::spawn(async move {
            process(socket, &s).await;
        });
    }
}

async fn process(socket: TcpStream, storage: &Storage) {
    let mut frame = Framed::new(socket, Codec);
    loop {
        match frame.next().await {
            Some(Ok(cmd)) => {
                let res = cmd.execute(storage);
                match res {
                    Ok(resp) => {
                        if let Err(e) = frame.send(resp).await {
                            error!("Error: {:?}", e);
                        }
                    }
                    Err(e) => error!("Error: {:?}", e),
                }
            }
            Some(Err(e)) => {
                info!("Error: {:?}", e);
            }
            None => {
                info!("Connection closed");
                break;
            }
        }
    }
}
