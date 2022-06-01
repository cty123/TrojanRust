use crate::config::base::{InboundConfig, OutboundConfig};
use crate::proxy::tcp::acceptor::Acceptor;
use crate::proxy::tcp::handler::Handler;
use log::{info, warn};
use std::io::Result;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use tokio::net::TcpListener;

pub async fn start(inbound_config: InboundConfig, outbound_config: OutboundConfig) -> Result<()> {
    // Extract the inbound client address
    let address = (inbound_config.address.clone(), inbound_config.port)
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();

    // Start the TCP server listener socket
    let listener = TcpListener::bind(address).await?;

    // Create TCP server acceptor and handler
    let (acceptor, handler) = (
        Arc::new(Acceptor::new(&inbound_config)),
        Arc::new(Handler::new(&outbound_config).unwrap()),
    );

    // Enter server listener socket accept loop
    loop {
        let (socket, addr) = listener.accept().await?;

        info!("Received new connection from {}", addr);

        let (acceptor, handler) = (acceptor.clone(), handler.clone());

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
