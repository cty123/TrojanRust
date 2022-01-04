use std::io::{Error, ErrorKind, Result};
use std::sync::Arc;

use log::info;
use rustls::{ClientConfig, ServerName};
use sha2::{Digest, Sha224};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_rustls::TlsConnector;

use crate::config::base::OutboundConfig;
use crate::config::tls::make_client_config;
use crate::protocol::common::request::{InboundRequest, TransportProtocol};
use crate::protocol::common::stream::{InboundStream, OutboundStream};
use crate::protocol::direct::outbound::DirectOutboundStream;
use crate::protocol::trojan::base::HEX_SIZE;
use crate::protocol::trojan::outbound::TrojanOutboundStream;
use crate::protocol::trojan::packet::PacketTrojanOutboundStream;
use crate::proxy::base::SupportedProtocols;
use crate::transport::grpc::proxy_service_client::ProxyServiceClient;
use crate::transport::grpc::GrpcDataOutboundStream;

/// Handler is responsible for taking user's request and process them and send back the result.
/// It may need to dial to remote using TCP, UDP and TLS, in which it will be responsible for
/// establishing a tranport level connection and escalate it to application data stream.
pub struct Handler {
    addr_port: Option<(String, u16)>,
    protocol: SupportedProtocols,
    tls_config: Option<Arc<ClientConfig>>,
    host_name: Option<String>,
    secret: Vec<u8>,
    transport: Option<TransportProtocol>,
}

impl Handler {
    /// Instantiate a new Handler instance based on OutboundConfig passed by the user. It will evaluate the
    /// TLS option particularly to be able to later determine whether it should escalate the connection to
    /// TLS first or not.
    pub fn new(outbound: OutboundConfig) -> Result<Handler> {
        // Get outbound TLS configuration and host dns name if TLS is enabled
        let (tls_config, host_name) = match outbound.tls {
            Some(cfg) => (Some(make_client_config(&cfg)), Some(cfg.host_name)),
            None => (None, None),
        };

        // Attempt to extract destination address and port from OutboundConfig.
        let addr_port = match (outbound.address, outbound.port) {
            (Some(addr), Some(port)) => Some((addr, port)),
            (Some(_), None) => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "Missing port while address is present",
                ))
            }
            (None, Some(_)) => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "Missing address while port is present",
                ))
            }
            // No destination address and port specified, will use the address and port in each request
            (None, None) => None,
        };

        // Extract the plaintext of the secret and process it
        let secret = match outbound.protocol {
            SupportedProtocols::TROJAN if outbound.secret.is_some() => {
                let secret = outbound.secret.as_ref().unwrap();
                Sha224::digest(secret.as_bytes())
                    .iter()
                    .map(|x| format!("{:02x}", x))
                    .collect::<String>()
                    .as_bytes()
                    .to_vec()
            }
            // Configure secret if need to add other protocols
            _ => Vec::new(),
        };

        Ok(Handler {
            protocol: outbound.protocol,
            addr_port,
            tls_config,
            host_name,
            secret,
            transport: outbound.transport,
        })
    }

    /// Given an abstract inbound stream, it will read the request to standard request format and then process it.
    /// After taking the request, the handler will then establish the outbound connection based on the user configuration,
    /// and transport data back and forth until one side terminate the connection.
    pub async fn dispatch(
        &self,
        inbound_stream: Box<dyn InboundStream>,
        request: InboundRequest,
    ) -> Result<()> {
        let outbound_stream = match self.transport {
            Some(_transport) if matches!(TransportProtocol::GRPC, _transport) => {
                self.handle_grpc(request).await?
            }
            _ => self.handle(&request).await?,
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

    /// Given an inbound request, handle the request by establishing a new TCP/UDP/TLS connection based on inbound
    /// handler configuration. If the connection is TCP, will also check if should escalate the connection to TLS.
    async fn handle(&self, request: &InboundRequest) -> Result<Box<dyn OutboundStream>> {
        return match request.transport_protocol {
            TransportProtocol::TCP => {
                let (addr, port) = match self.addr_port.clone() {
                    Some(addr_port) => addr_port,
                    None => request.addr_port(),
                };

                // Establish raw TCP connection with remote
                let connection = match TcpStream::connect((addr.as_ref(), port)).await {
                    Ok(connection) => {
                        info!("Established connection to remote host at {}:{}", addr, port);
                        connection
                    }
                    Err(e) => {
                        return Err(Error::new(
                            ErrorKind::ConnectionReset,
                            format!("Failed to connect to {}:{}, {}", addr, port, e),
                        ));
                    }
                };

                // Escalate raw TCP connection to TLS
                return match (self.tls_config.as_ref(), self.host_name.as_ref()) {
                    (Some(tls), Some(hname)) => {
                        let connector = TlsConnector::from(Arc::clone(tls));
                        let domain = match ServerName::try_from(hname.as_ref()) {
                            Ok(domain) => domain,
                            Err(_) => return Err(Error::new(ErrorKind::InvalidInput, "Failed to parse host name"))
                        };
                        let tls_stream = connector.connect(domain, connection).await?;
                        self.handle_protocol(tls_stream, request).await
                    },
                    (Some(_), None) => {
                        return Err(Error::new(
                            ErrorKind::InvalidInput,
                            "Failed to find destination address, destination port or host name from configuration",
                        ))
                    },
                    (None, _) => self.handle_protocol(connection, request).await
                };
            }
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
            TransportProtocol::GRPC => Err(Error::new(
                ErrorKind::ConnectionReset,
                "unsupported protocol",
            )),
        };
    }

    async fn handle_grpc(&self, request: InboundRequest) -> Result<Box<dyn OutboundStream>> {
        let (addr, port) = match self.addr_port.clone() {
            Some(addr_port) => addr_port,
            None => request.addr_port(),
        };

        let endpoint =
            match tonic::transport::Channel::from_shared(format!("http://{}:{}", addr, port)) {
                Ok(endpoint) => endpoint,
                Err(e) => return Err(Error::new(ErrorKind::AddrNotAvailable, e)),
            };

        let channel = match endpoint.connect().await {
            Ok(channel) => channel,
            Err(e) => return Err(Error::new(ErrorKind::ConnectionReset, e)),
        };

        let mut client = ProxyServiceClient::new(channel);
        let (tx, rx) = mpsc::channel(64);
        let read_half = match client
            .proxy(tokio_stream::wrappers::ReceiverStream::new(rx))
            .await
        {
            Ok(response) => response.into_inner(),
            Err(e) => return Err(Error::new(ErrorKind::Interrupted, e)),
        };

        return match TrojanOutboundStream::new(
            GrpcDataOutboundStream::new(read_half, tx),
            &request,
            &self.secret,
        )
        .await
        {
            Ok(stream) => Ok(stream),
            Err(e) => return Err(Error::new(ErrorKind::BrokenPipe, e.to_string())),
        };
    }

    #[inline]
    async fn handle_protocol<IO>(
        &self,
        stream: IO,
        request: &InboundRequest,
    ) -> Result<Box<dyn OutboundStream>>
    where
        IO: AsyncRead + AsyncWrite + Unpin + Sync + Send + 'static,
    {
        return match self.protocol {
            SupportedProtocols::DIRECT => Ok(DirectOutboundStream::new(stream)),
            SupportedProtocols::TROJAN => {
                if self.secret.len() != HEX_SIZE {
                    return Err(Error::new(
                        ErrorKind::InvalidInput,
                        format!("Hex in trojan protocol is not {} bytes", HEX_SIZE),
                    ));
                }
                let outbound_stream =
                    TrojanOutboundStream::new(stream, request, &self.secret).await?;
                Ok(outbound_stream)
            }
            SupportedProtocols::SOCKS => {
                Err(Error::new(ErrorKind::Unsupported, "Unsupported protocol"))
            }
        };
    }
}
