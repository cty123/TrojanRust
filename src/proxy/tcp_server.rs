use std::io::Result;
use std::sync::Arc;

use log::{info, warn};
use rustls::ServerConfig;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

use crate::config::base::{InboundConfig, OutboundConfig};
use crate::config::tls::make_server_config;
use crate::protocol::common::stream::InboundStream;
use crate::proxy::acceptor::Acceptor;
use crate::proxy::base::SupportedProtocols;
use crate::proxy::handler::Handler;

pub struct TcpServer {
    local_addr: String,
    local_port: u16,
    protocol: SupportedProtocols,
    tls: bool,
    tls_config: Option<Arc<ServerConfig>>,
    handler: Handler,
}

impl TcpServer {
    pub fn new(
        inbound_config: InboundConfig,
        outbound_config: OutboundConfig,
    ) -> Result<TcpServer> {
        let handler = Handler::new(outbound_config);

        return Ok(TcpServer {
            local_addr: inbound_config.address,
            local_port: inbound_config.port,
            protocol: inbound_config.protocol,
            tls: inbound_config.tls.is_some(),
            tls_config: make_server_config(inbound_config.tls),
            handler,
        });
    }

    pub async fn start(self) -> Result<()> {
        let listener =
            TcpListener::bind(format!("{}:{}", self.local_addr, self.local_port)).await?;

        let acceptor = match self.tls_config {
            Some(config) => Acceptor::new(
                self.protocol,
                self.local_port,
                Some(TlsAcceptor::from(config)),
            ),
            None => Acceptor::new(self.protocol, self.local_port, None),
        };

        info!(
            "TCP server started on {}:{}, ready to accept input stream",
            self.local_addr, self.local_port
        );

        loop {
            let (socket, addr) = listener.accept().await?;

            info!("Received new connection from {}", addr);

            let acceptor = acceptor.clone();
            let handler = self.handler.clone();

            tokio::spawn(async move {
                let mut inbound_stream = match acceptor.accept(socket).await {
                    Ok(stream) => stream,
                    Err(e) => {
                        warn!("Failed to accept inbound connection from {}: {}", addr, e);
                        return;
                    }
                };
                match TcpServer::dispatch(&mut inbound_stream, handler).await {
                    Ok(_) => {
                        info!("Connection to {} has finished", addr);
                    }
                    Err(e) => {
                        warn!("Failed to handle the inbound stream: {}", e);
                    }
                }
            });
        }
    }

    async fn dispatch(
        inbound_stream: &mut Box<dyn InboundStream>,
        outbound_handler: Handler,
    ) -> Result<()> {
        let request = inbound_stream.handshake().await?;

        let outbound_stream = match outbound_handler.handle(&request).await {
            Ok(stream) => stream,
            Err(e) => return Err(e),
        };

        let (mut source_read, mut source_write) = tokio::io::split(inbound_stream);
        let (mut target_read, mut target_write) = tokio::io::split(outbound_stream);

        return match futures::future::join(
            tokio::io::copy(&mut source_read, &mut target_write),
            tokio::io::copy(&mut target_read, &mut source_write),
        )
        .await
        {
            (Err(e), _) | (_, Err(e)) => Err(e),
            _ => Ok(()),
        };
    }
}
