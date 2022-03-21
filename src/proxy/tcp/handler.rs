use crate::config::base::OutboundConfig;
use crate::config::tls::make_client_config;
use crate::protocol::common::request::{InboundRequest, TransportProtocol};
use crate::protocol::common::stream::{StandardStream, StandardTcpStream};
use crate::protocol::trojan;
use crate::protocol::trojan::handshake;
use crate::protocol::trojan::HEX_SIZE;
use crate::proxy::base::SupportedProtocols;
use crate::proxy::grpc::client::{handle_client_data, handle_server_data};
use crate::transport::grpc::proxy_service_client::ProxyServiceClient;
use crate::transport::grpc::{GrpcPacket, TrojanRequest};

use log::{error, info};
use rustls::ServerName;
use sha2::{Digest, Sha224};
use std::io::{Error, ErrorKind, Result};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{TcpStream, UdpSocket};
use tokio::sync::mpsc;
use tokio_rustls::TlsConnector;

/// Handler is responsible for taking user's request and process them and send back the result.
/// It may need to dial to remote using TCP, UDP and TLS, in which it will be responsible for
/// establishing a tranport level connection and escalate it to application data stream.
pub struct Handler {
    destination: Option<SocketAddr>,
    protocol: SupportedProtocols,
    tls: Option<(TlsConnector, ServerName)>,
    // grpc_channel: Option<Endpoint>,
    secret: Vec<u8>,
    transport: Option<TransportProtocol>,
}

impl Handler {
    /// Instantiate a new Handler instance based on OutboundConfig passed by the user. It will evaluate the
    /// TLS option particularly to be able to later determine whether it should escalate the connection to
    /// TLS first or not.
    pub fn new(outbound: &OutboundConfig) -> Result<Handler> {
        // Get outbound TLS configuration and host dns name if TLS is enabled
        let tls = match &outbound.tls {
            Some(cfg) => {
                let tls_client_config = make_client_config(&cfg);
                let connector = TlsConnector::from(tls_client_config);
                let domain = match ServerName::try_from(cfg.host_name.as_ref()) {
                    Ok(domain) => domain,
                    Err(_) => {
                        return Err(Error::new(
                            ErrorKind::InvalidInput,
                            "Failed to parse host name",
                        ))
                    }
                };
                Some((connector, domain))
            }
            None => None,
        };

        // Attempt to extract destination address and port from OutboundConfig.
        let destination = match (outbound.address.clone(), outbound.port) {
            (Some(addr), Some(port)) => Some(format!("{}:{}", addr, port).parse().unwrap()),
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
                let secret = outbound.secret.clone().unwrap();
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
            destination,
            // grpc_channel,
            tls,
            secret,
            transport: outbound.transport,
        })
    }

    /// Given an abstract inbound stream, it will read the request to standard request format and then process it.
    /// After taking the request, the handler will then establish the outbound connection based on the user configuration,
    /// and transport data back and forth until one side terminate the connection.
    pub async fn dispatch<T: AsyncRead + AsyncWrite + Unpin + Send + 'static>(
        &self,
        inbound_stream: StandardStream<StandardTcpStream<T>>,
        request: InboundRequest,
    ) -> Result<()> {
        match request.transport_protocol {
            TransportProtocol::TCP if self.transport.is_some() => {
                self.handle_grpc_stream(request, inbound_stream).await?
            }
            TransportProtocol::TCP => self.handle_byte_stream(request, inbound_stream).await?,
            TransportProtocol::UDP => self.handle_packet_stream(request, inbound_stream).await?,
            _ => return Ok(()),
        }

        Ok(())
    }

    async fn handle_grpc_stream<T: AsyncRead + AsyncWrite + Unpin + Send + 'static>(
        &self,
        request: InboundRequest,
        inbound_stream: StandardStream<StandardTcpStream<T>>,
    ) -> Result<()> {
        let mut server = ProxyServiceClient::connect(format!(
            "http://{}",
            self.destination.unwrap().to_string()
        ))
        .await
        .unwrap();

        let (tx, rx) = mpsc::channel(64);

        match tx
            .send(GrpcPacket {
                packet_type: 1,
                trojan: Some(TrojanRequest {
                    hex: self.secret.clone(),
                    atype: request.atype.to_byte() as u32,
                    command: request.command.to_byte() as u32,
                    address: request.addr.to_string(),
                    port: request.port as u32,
                }),
                datagram: None,
            })
            .await
        {
            Ok(_) => (),
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::ConnectionRefused,
                    "failed to write request data",
                ))
            }
        }

        let server_reader = match server
            .proxy(tokio_stream::wrappers::ReceiverStream::new(rx))
            .await
        {
            Ok(response) => response.into_inner(),
            Err(e) => return Err(Error::new(ErrorKind::Interrupted, e)),
        };

        let (client_reader, client_writer) = tokio::io::split(inbound_stream.into_inner());

        return match tokio::try_join!(
            tokio::spawn(handle_server_data(client_reader, tx)),
            tokio::spawn(handle_client_data(client_writer, server_reader))
        ) {
            Ok(_) => {
                info!("Connection finished");
                Ok(())
            }
            Err(e) => {
                error!("Encountered error while handling the transport: {}", e);
                Err(Error::new(ErrorKind::ConnectionReset, "Connection reset"))
            }
        };
    }

    /// Handle TCP like byte stream and finish data transfer
    async fn handle_byte_stream<T: AsyncRead + AsyncWrite + Unpin + Send + 'static>(
        &self,
        request: InboundRequest,
        inbound_stream: StandardStream<StandardTcpStream<T>>,
    ) -> Result<()> {
        let outbound_stream = self.dial_tcp_outbound(request).await?;

        let (mut source_read, mut source_write) = tokio::io::split(inbound_stream.into_inner());
        let (mut target_read, mut target_write) = tokio::io::split(outbound_stream.into_inner());

        return match tokio::try_join!(
            tokio::spawn(async move {
                return tokio::io::copy(&mut source_read, &mut target_write).await;
            }),
            tokio::spawn(async move {
                return tokio::io::copy(&mut target_read, &mut source_write).await;
            }),
        ) {
            Ok(_) => {
                info!("Connection finished");
                Ok(())
            }
            Err(e) => {
                error!("Encountered error while handling the transport: {}", e);
                Err(Error::new(ErrorKind::ConnectionReset, "Connection reset"))
            }
        };
    }

    async fn dial_tcp_outbound(
        &self,
        request: InboundRequest,
    ) -> Result<StandardStream<StandardTcpStream<TcpStream>>> {
        let dest = match self.destination {
            Some(dest) => dest,
            None => request.into_destination_address(),
        };

        let connection = TcpStream::connect(dest).await?;
        let connection = match &self.tls {
            Some((connector, domain)) => {
                let connection = connector.connect(domain.clone(), connection).await?;
                StandardTcpStream::RustlsClient(connection)
                // self.handle_tcp_outbound(connection, request).await
            }
            None => StandardTcpStream::Plain(connection),
        };

        self.handle_tcp_outbound(connection, request).await
    }

    async fn handle_packet_stream<T: AsyncRead + AsyncWrite + Unpin>(
        &self,
        request: InboundRequest,
        inbound_stream: StandardStream<StandardTcpStream<T>>,
    ) -> Result<()> {
        // Establish UDP connection to remote host
        let socket = match self.dial_udp_outbound(&request).await {
            Ok(s) => Arc::new(s),
            Err(e) => return Err(e),
        };

        // Setup the reader and writer for both the client and server so that we can transport the data
        let (mut client_reader, mut client_writer) = tokio::io::split(inbound_stream.into_inner());
        let (server_reader, server_writer) = (socket.clone(), socket.clone());

        // Assume the protocol is always trojan
        return match tokio::try_join!(
            trojan::handle_client_data(&mut client_reader, &server_writer),
            trojan::handle_server_data(&mut client_writer, &server_reader, request)
        ) {
            Ok(_) => {
                info!("Connection finished");
                Ok(())
            }
            Err(e) => {
                error!("Encountered {} error while handing the transport", e);
                Err(Error::new(ErrorKind::ConnectionReset, "Connection reset"))
            }
        };
    }

    async fn dial_udp_outbound(&self, request: &InboundRequest) -> Result<UdpSocket> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect(request.into_destination_address()).await?;
        Ok(socket)
    }

    #[inline]
    async fn handle_tcp_outbound<T: AsyncRead + AsyncWrite + Unpin>(
        &self,
        stream: T,
        request: InboundRequest,
    ) -> Result<StandardStream<T>> {
        return match self.protocol {
            SupportedProtocols::DIRECT => Ok(StandardStream::new(stream)),
            SupportedProtocols::TROJAN => {
                if self.secret.len() != HEX_SIZE {
                    return Err(Error::new(
                        ErrorKind::InvalidInput,
                        format!("Hex in trojan protocol is not {} bytes", HEX_SIZE),
                    ));
                }
                Ok(handshake(stream, request, &self.secret).await?)
            }
            SupportedProtocols::SOCKS => {
                Err(Error::new(ErrorKind::Unsupported, "Unsupported protocol"))
            }
        };
    }
}
