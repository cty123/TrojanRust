use std::io::{Error, ErrorKind, Result};

use log::{info, warn};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;

use crate::config::base::OutboundConfig;
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

    async fn tls(addr_port: &str) {}

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
