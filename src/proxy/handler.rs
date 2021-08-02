use std::io::{Error, ErrorKind, Result};
use std::sync::Arc;

use log::{info, warn};
use rustls::ClientConfig;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio_rustls::{webpki::DNSNameRef, TlsConnector};

use crate::config::base::OutboundConfig;
use crate::config::tls::get_client_config;
use crate::protocol::common::request::{InboundRequest, TransportProtocol};
use crate::protocol::common::stream::OutboundStream;
use crate::protocol::direct::outbound::DirectOutboundStream;
use crate::protocol::trojan::outbound::TrojanOutboundStream;
use crate::protocol::trojan::packet::PacketTrojanOutboundStream;
use crate::proxy::base::SupportedProtocols;

#[derive(Clone)]
pub struct Handler {
    protocol: SupportedProtocols,
    tls: bool,
    tls_config: Option<Arc<ClientConfig>>,
    destination: Option<String>,
}

impl Handler {
    pub fn new(config: OutboundConfig) -> Handler {
        let destination = if config.address.is_some() && config.port.is_some() {
            Some(format!(
                "{}:{}",
                config.address.unwrap(),
                config.port.unwrap()
            ))
        } else {
            None
        };

        Handler {
            protocol: config.protocol,
            destination,
            tls: false,
            tls_config: get_client_config(config.tls, config.insecure),
        }
    }

    pub async fn handle(self, request: &InboundRequest) -> Result<Box<dyn OutboundStream>> {
        let addr_port = request.addr_port();

        return match request.transport_protocol {
            TransportProtocol::TCP => self.tcp_dial(request).await,
            TransportProtocol::UDP => {
                info!("UDP associate to {}", request.addr_port());
                match PacketTrojanOutboundStream::new().await {
                    Ok(c) => Ok(c),
                    Err(_) => Err(Error::new(
                        ErrorKind::ConnectionReset,
                        format!("Failed to dial udp connection to {}", addr_port),
                    )),
                }
            }
        };
    }

    #[inline]
    async fn tcp_dial(self, request: &InboundRequest) -> Result<Box<dyn OutboundStream>> {
        return match self.tls {
            true => Err(Error::new(ErrorKind::Unsupported, "Unsupported")),
            false if self.destination.is_some() => {
                Handler::tcp(&self.destination.unwrap(), request, self.protocol).await
            }
            false => Handler::tcp(&request.addr_port(), request, self.protocol).await,
        };
    }

    async fn tcp(
        addr_port: &str,
        request: &InboundRequest,
        protocol: SupportedProtocols,
    ) -> Result<Box<dyn OutboundStream>> {
        info!("Dialing remote host at {}", addr_port);
        let connection = match TcpStream::connect(addr_port).await {
            Ok(connection) => connection,
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::ConnectionReset,
                    format!("Failed to connect to {}: {}", addr_port, e),
                ))
            }
        };

        return Handler::handle_protocol(connection, protocol, request).await;
    }

    async fn tls(
        addr_port: &str,
        request: &InboundRequest,
        config: Option<Arc<ClientConfig>>,
        protocol: SupportedProtocols,
    ) -> Result<Box<dyn OutboundStream>> {
        return match config {
            Some(cfg) => {
                let config = TlsConnector::from(cfg);
                let stream = TcpStream::connect(&addr_port).await?;
                let domain = DNSNameRef::try_from_ascii_str("example.com")
                    .map_err(|_| Error::new(ErrorKind::InvalidInput, "invalid dnsname"))?;
                let tls_stream = config.connect(domain, stream).await?;
                Handler::handle_protocol(tls_stream, protocol, request).await
            }
            None => Err(Error::new(ErrorKind::InvalidInput, "No tls config")),
        };
    }

    async fn handle_protocol<IO>(
        stream: IO,
        protocol: SupportedProtocols,
        request: &InboundRequest,
    ) -> Result<Box<dyn OutboundStream>>
    where
        IO: AsyncRead + AsyncWrite + Unpin + Sync + Send + 'static,
    {
        return match protocol {
            SupportedProtocols::DIRECT => Ok(DirectOutboundStream::new(stream)),
            SupportedProtocols::TROJAN => Ok(TrojanOutboundStream::new(stream, request)),
            SupportedProtocols::SOCKS => Err(Error::new(ErrorKind::Unsupported, "Unsupported")),
        };
    }
}
