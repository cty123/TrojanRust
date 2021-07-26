use log::{error, info};

use std::io::{Error, ErrorKind, Result};
use std::sync::Arc;

use tokio::net::TcpListener;

use rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;

use crate::config::base::{InboundConfig, OutboundConfig};
use crate::config::tls::get_tls_config;
use crate::protocol::common::stream::InboundStream;
use crate::proxy::acceptor::Acceptor;
use crate::proxy::base::SupportedProtocols;
use crate::proxy::handler::Handler;

pub struct TcpServer {
    local_addr: String,
    local_port: u16,
    protocol: SupportedProtocols,
    tls_config: Option<Arc<ServerConfig>>,
    handler: Handler,
}

impl TcpServer {
    pub fn new(
        inbound_config: InboundConfig,
        outbound_config: OutboundConfig,
    ) -> Result<TcpServer> {
        let tls_config = match inbound_config.tls {
            true if inbound_config.cert_path.is_some() && inbound_config.key_path.is_some() => {
                Some(get_tls_config(
                    &inbound_config.cert_path.unwrap(),
                    &inbound_config.key_path.unwrap(),
                )?)
            }
            true => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "missing tls certificate path or tls private key path",
                ))
            }
            false => None,
        };

        let handler = Handler::new();

        return Ok(TcpServer {
            local_addr: inbound_config.address,
            local_port: inbound_config.port,
            protocol: inbound_config.protocol,
            tls_config,
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
            let (socket, _) = listener.accept().await?;
            let acceptor = acceptor.clone();
            let mut inbound_stream = acceptor.accept(socket).await?;

            let handler = self.handler.clone();

            tokio::spawn(async move {
                match TcpServer::dispatch(&mut inbound_stream, handler).await {
                    Ok(_) => {
                        info!("Connection finished");
                        Ok(())
                    },
                    Err(e) => {
                        error!("Failed to handle the inbound stream: {}", e);
                        return Err(e);
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

        let outbound_stream = match outbound_handler.handle(request).await {
            Some(stream) => stream,
            None => {
                return Err(Error::new(
                    ErrorKind::ConnectionReset,
                    "Unable to establish connection to remote",
                ))
            }
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
