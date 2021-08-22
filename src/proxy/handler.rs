use std::io::{Error, ErrorKind, Result};
use std::sync::Arc;

use log::{info, warn};
use rustls::ClientConfig;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio_rustls::{webpki::DNSNameRef, TlsConnector};

use crate::config::base::OutboundConfig;
use crate::config::tls::make_client_config;
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
    addr: Option<String>,
    port: Option<u16>,
}

impl Handler {
    pub fn new(config: OutboundConfig) -> Handler {
        Handler {
            protocol: config.protocol,
            addr: config.address,
            port: config.port,
            tls: config.tls.is_some(),
            tls_config: make_client_config(config.tls),
        }
    }

    #[inline]
    pub async fn handle(self, request: &InboundRequest) -> Result<Box<dyn OutboundStream>> {
        return match request.transport_protocol {
            TransportProtocol::TCP => self.tcp_dial_destination(request).await,
            TransportProtocol::UDP => {
                let (addr, port) = request.addr_port();
                info!("Handle UDP associate to {}:{}", addr, port);
                match PacketTrojanOutboundStream::new().await {
                    Ok(c) => Ok(c),
                    Err(e) => Err(Error::new(
                        ErrorKind::ConnectionReset,
                        format!("Failed to dial udp connection to {}:{}, {}", addr, port, e),
                    )),
                }
            }
        };
    }

    #[inline]
    async fn tcp_dial_destination(
        self,
        request: &InboundRequest,
    ) -> Result<Box<dyn OutboundStream>> {
        return match self.tls {
            true if self.tls_config.is_some() && self.addr.is_some() && self.port.is_some() => {
                Handler::tls(
                    &self.addr.unwrap(),
                    self.port.unwrap(),
                    request,
                    self.tls_config.unwrap(),
                    self.protocol,
                )
                .await
            }
            true => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "Failed to find tls config",
                ))
            }
            false if self.addr.is_some() => {
                Handler::tcp(
                    &self.addr.unwrap(),
                    self.port.unwrap(),
                    request,
                    self.protocol,
                )
                .await
            }
            false => {
                let (addr, port) = request.addr_port();
                Handler::tcp(&addr, port, request, self.protocol).await
            }
        };
    }

    #[inline]
    async fn tcp(
        addr: &str,
        port: u16,
        request: &InboundRequest,
        protocol: SupportedProtocols,
    ) -> Result<Box<dyn OutboundStream>> {
        info!("Dialing remote host at {}:{}", addr, port);
        let connection = match TcpStream::connect((addr, port)).await {
            Ok(connection) => connection,
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::ConnectionReset,
                    format!("Failed to connect to {}:{}, {}", addr, port, e),
                ))
            }
        };

        return Handler::handle_protocol(connection, protocol, request).await;
    }

    #[inline]
    async fn tls(
        addr: &str,
        port: u16,
        request: &InboundRequest,
        config: Arc<ClientConfig>,
        protocol: SupportedProtocols,
    ) -> Result<Box<dyn OutboundStream>> {
        let config = TlsConnector::from(config);
        let stream = TcpStream::connect((addr, port)).await?;
        let domain = DNSNameRef::try_from_ascii_str("example.com")
            .map_err(|_| Error::new(ErrorKind::InvalidInput, "Invalid dnsname"))?;
        let tls_stream = config.connect(domain, stream).await?;
        Handler::handle_protocol(tls_stream, protocol, request).await
    }

    #[inline]
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
