use std::io::Result;
use std::sync::Arc;

use log::{info, warn};
use tokio::net::TcpListener;

use crate::config::base::{InboundConfig, OutboundConfig};
use crate::proxy::acceptor::Acceptor;
use crate::proxy::handler::Handler;

pub struct TcpServer {
    local_addr: String,
    local_port: u16,
    acceptor: Arc<Acceptor>,
    handler: Arc<Handler>,
}

impl TcpServer {
    pub fn new(
        inbound_config: InboundConfig,
        outbound_config: OutboundConfig,
    ) -> Result<TcpServer> {
        let handler = Arc::from(Handler::new(outbound_config)?);
        let acceptor = Arc::from(Acceptor::new(&inbound_config));

        return Ok(TcpServer {
            local_addr: inbound_config.address,
            local_port: inbound_config.port,
            handler,
            acceptor,
        });
    }

    pub async fn start(self) -> Result<()> {
        let (local_addr, local_port) = (self.local_addr, self.local_port);

        let listener = TcpListener::bind((local_addr.as_ref(), local_port)).await?;

        info!(
            "TCP server started on {}:{}, ready to accept input stream",
            local_addr, local_port
        );

        loop {
            let (socket, addr) = listener.accept().await?;

            info!("Received new connection from {}", addr);

            let acceptor = Arc::clone(&self.acceptor);
            let handler = Arc::clone(&self.handler);

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
}
