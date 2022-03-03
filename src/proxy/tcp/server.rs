use crate::proxy::tcp::acceptor::Acceptor;
use crate::proxy::tcp::handler::Handler;
use log::{info, warn};
use std::io::Result;
use std::net::SocketAddr;
use tokio::net::TcpListener;

pub async fn start(
    address: SocketAddr,
    acceptor: &'static Acceptor,
    handler: &'static Handler,
) -> Result<()> {
    let listener = TcpListener::bind(address).await?;

    loop {
        let (socket, addr) = listener.accept().await?;

        info!("Received new connection from {}", addr);

        tokio::spawn(async move {
            let (request, inbound_stream) = match acceptor.accept(socket).await {
                Ok(stream) => stream,
                Err(e) => {
                    warn!("Failed to accept inbound connection from {}: {}", addr, e);
                    return;
                }
            };

            match handler.dispatch(inbound_stream, request).await {
                Ok(_) => {
                    info!("Connection from {} has finished", addr);
                }
                Err(e) => {
                    warn!("Failed to handle the inbound stream: {}", e);
                }
            }
        });
    }
}
