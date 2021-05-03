use log::{info, warn};
use tokio::net::TcpListener;
// use tokio::io::{AsyncReadExt, AsyncWriteExt};

mod transport;
mod protocol;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Initalize configurations
    env_logger::init();
    info!("Rust starting");

    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            match transport::tcp::dispatch(socket).await {
                Ok(_) => {
                    info!("Finished processing socket");
                },
                Err(e) => {
                    warn!("Error in dispatching the TCP socket: {}", e);
                }
            }
        });
    }
}