use crate::config::base::{InboundConfig, OutboundConfig};
use crate::proxy::tcp::acceptor::TcpAcceptor;
use crate::proxy::tcp::handler::TcpHandler;

use log::{info, warn};
use std::io::Result;
use std::net::{SocketAddr, ToSocketAddrs};
use tokio::net::TcpListener;

/// Start running raw TCP server
pub async fn start(
    inbound_config: &'static InboundConfig,
    outbound_config: &'static OutboundConfig,
) -> Result<()> {
    // Extract the inbound config address
    let addresses: Vec<SocketAddr> = (inbound_config.address.clone(), inbound_config.port)
        .to_socket_addrs()
        .unwrap()
        .collect();

    // Start the TCP server listener socket
    let listener = TcpListener::bind(&addresses[..]).await?;

    // Create TCP server acceptor and handler
    let (acceptor, handler) = (
        TcpAcceptor::init(&inbound_config),
        TcpHandler::init(&outbound_config),
    );

    // Enter server listener socket accept loop
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
