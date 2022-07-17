use crate::config::base::{OutboundConfig, OutboundMode};
use crate::config::tls::{make_client_config, NoCertificateVerification};
use crate::protocol::common::request::{InboundRequest, TransportProtocol};
use crate::protocol::common::stream::StandardTcpStream;
use crate::protocol::trojan;
use crate::protocol::trojan::handshake;
use crate::protocol::trojan::packet::{
    packet_reader_to_stream_writer, stream_reader_to_packet_writer, TrojanPacketReader,
    TrojanPacketWriter,
};
use crate::protocol::trojan::HEX_SIZE;
use crate::proxy::base::SupportedProtocols;

use log::info;
use once_cell::sync::OnceCell;
use rustls::{ClientConfig, ServerName};
use sha2::{Digest, Sha224};
use std::io::{Error, ErrorKind, Result};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{TcpStream, UdpSocket};
use tokio_rustls::TlsConnector;

/// Static life time TCP server outbound traffic handler to avoid ARC
/// The handler is initialized through init() function
static TCP_HANDLER: OnceCell<TcpHandler> = OnceCell::new();

/// Handler is responsible for taking user's request and process them and send back the result.
/// It may need to dial to remote using TCP, UDP and TLS, in which it will be responsible for
/// establishing a tranport level connection and escalate it to application data stream.
pub struct TcpHandler {
    mode: OutboundMode,
    protocol: SupportedProtocols,
    destination: Option<SocketAddr>,
    tls: Option<(Arc<ClientConfig>, ServerName)>,
    secret: Vec<u8>,
}

impl TcpHandler {
    /// Instantiate a new Handler instance based on OutboundConfig passed by the user. It will evaluate the
    /// TLS option particularly to be able to later determine whether it should escalate the connection to
    /// TLS first or not.
    pub fn init(outbound: &OutboundConfig) -> &'static TcpHandler {
        // Get outbound TLS configuration and host dns name if TLS is enabled
        let tls = match &outbound.tls {
            Some(cfg) => {
                let client_config = make_client_config(&cfg);
                Some((
                    client_config,
                    ServerName::try_from(cfg.host_name.as_ref())
                        .expect("Failed to parse host name"),
                ))
            }
            None => None,
        };

        // Attempt to extract destination address and port from OutboundConfig.
        let destination = match (outbound.address.clone(), outbound.port) {
            (Some(addr), Some(port)) => Some(format!("{}:{}", addr, port).parse().unwrap()),
            (Some(_), None) => {
                panic!("Missing port while address is present")
            }
            (None, Some(_)) => {
                panic!("Missing address while address is present")
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

        TCP_HANDLER.get_or_init(|| Self {
            mode: outbound.mode.clone(),
            protocol: outbound.protocol,
            destination,
            tls,
            secret,
        })
    }

    /// Given an abstract inbound stream, it will read the request to standard request format and then process it.
    /// After taking the request, the handler will then establish the outbound connection based on the user configuration,
    /// and transport data back and forth until one side terminate the connection.
    pub async fn dispatch<T: AsyncRead + AsyncWrite + Unpin + Send + 'static>(
        &self,
        inbound_stream: StandardTcpStream<T>,
        request: InboundRequest,
    ) -> Result<()> {
        match self.mode {
            OutboundMode::DIRECT => self.handle_direct_stream(request, inbound_stream).await?,
            OutboundMode::TCP => self.handle_tcp_stream(request, inbound_stream).await?,
            OutboundMode::QUIC => self.handle_quic_stream(request, inbound_stream).await?,
            OutboundMode::GRPC => {
                return Err(Error::new(
                    ErrorKind::Unsupported,
                    "GRPC transport outbound is not yet supported",
                ))
            }
        }

        Ok(())
    }

    /// Handle direct data transport without any proxy protocol
    async fn handle_direct_stream<T: AsyncRead + AsyncWrite + Unpin + Send>(
        &self,
        request: InboundRequest,
        inbound_stream: StandardTcpStream<T>,
    ) -> Result<()> {
        let addr = request.into_destination_address();

        match request.proxy_protocol {
            SupportedProtocols::SOCKS => todo!(),
            SupportedProtocols::TROJAN => {
                match request.transport_protocol {
                    TransportProtocol::TCP => {
                        // Connect to remote server from the proxy request
                        let outbound_stream = match TcpStream::connect(addr).await {
                            Ok(stream) => stream,
                            Err(e) => {
                                return Err(Error::new(
                                    ErrorKind::ConnectionRefused,
                                    format!("failed to connect to tcp {}: {}", addr, e),
                                ))
                            }
                        };

                        // Setup the reader and writer for both the client and server so that we can transport the data
                        let (mut client_reader, mut client_writer) =
                            tokio::io::split(inbound_stream);
                        let (mut server_reader, mut server_writer) =
                            tokio::io::split(outbound_stream);

                        tokio::select!(
                            _ = tokio::io::copy(&mut client_reader, &mut server_writer) => (),
                            _ = tokio::io::copy(&mut server_reader, &mut client_writer) => (),
                        );
                    }
                    TransportProtocol::UDP => {
                        // Establish UDP connection to remote host
                        let socket = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
                        match socket.connect(addr).await {
                            Ok(_) => (),
                            Err(e) => {
                                return Err(Error::new(
                                    ErrorKind::ConnectionRefused,
                                    format!("failed to connect to udp {}: {}", addr, e),
                                ))
                            }
                        }

                        // Setup the reader and writer for both the client and server so that we can transport the data
                        let (server_reader, server_writer) = (socket.clone(), socket.clone());
                        let (client_reader, client_writer) = tokio::io::split(inbound_stream);
                        let (client_packet_reader, client_packet_writer) = (
                            TrojanPacketReader::new(client_reader),
                            TrojanPacketWriter::new(client_writer, request),
                        );

                        tokio::select!(
                            _ = trojan::packet::packet_reader_to_udp_packet_writer(client_packet_reader, server_writer) => (),
                            _ = trojan::packet::udp_packet_reader_to_packet_writer(server_reader, client_packet_writer) => ()
                        );
                    }
                };
            }
            SupportedProtocols::DIRECT => {
                return Err(Error::new(
                    ErrorKind::Unsupported,
                    "Proxy request can't have direct as proxy protocol",
                ))
            }
        };

        info!("Connection finished");

        Ok(())
    }

    async fn handle_quic_stream<T: AsyncRead + AsyncWrite + Unpin + Send + 'static>(
        &self,
        request: InboundRequest,
        inbound_stream: StandardTcpStream<T>,
    ) -> Result<()> {
        // Dial remote proxy server
        let _roots = rustls::RootCertStore::empty();
        let client_crypto = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_custom_certificate_verifier(Arc::new(NoCertificateVerification {}))
            .with_no_client_auth();
        let mut endpoint = quinn::Endpoint::client("[::]:0".parse().unwrap()).unwrap();
        endpoint.set_default_client_config(quinn::ClientConfig::new(Arc::new(client_crypto)));

        // Establish connection with remote proxy server using QUIC protocol
        let connection = endpoint
            .connect("127.0.0.1:8081".parse().unwrap(), "example.com")
            .unwrap()
            .await
            .unwrap();

        let quinn::NewConnection {
            connection: conn, ..
        } = connection;

        let (mut server_writer, mut server_reader) = conn.open_bi().await.unwrap();
        let (mut client_reader, mut client_writer) = tokio::io::split(inbound_stream);

        handshake(&mut server_writer, &request, &self.secret).await?;

        tokio::select!(
            _ = tokio::spawn(async move {tokio::io::copy(&mut client_reader, &mut server_writer).await}) => (),
            _ = tokio::spawn(async move {tokio::io::copy(&mut server_reader, &mut client_writer).await}) => (),
        );

        Ok(())
    }

    async fn handle_tcp_stream<T: AsyncRead + AsyncWrite + Unpin + Send>(
        &self,
        request: InboundRequest,
        inbound_stream: StandardTcpStream<T>,
    ) -> Result<()> {
        // Establish the initial connection with remote server
        let connection = match self.destination {
            Some(dest) => TcpStream::connect(dest).await?,
            None => {
                return Err(Error::new(
                    ErrorKind::NotConnected,
                    "missing address of the remote server",
                ))
            }
        };

        // Escalate the connection to TLS connection if tls config is present
        let mut outbound_stream = match &self.tls {
            Some((client_config, domain)) => {
                let connector = TlsConnector::from(client_config.clone());
                StandardTcpStream::RustlsClient(
                    connector.connect(domain.clone(), connection).await?,
                )
            }
            None => StandardTcpStream::Plain(connection),
        };

        // Handshake to form the proxy stream
        match self.protocol {
            SupportedProtocols::TROJAN => {
                // Check Trojan secret match
                if self.secret.len() != HEX_SIZE {
                    return Err(Error::new(
                        ErrorKind::InvalidInput,
                        format!("Hex in trojan protocol is not {} bytes", HEX_SIZE),
                    ));
                }

                // Start handshake to establish proxy stream
                handshake(&mut outbound_stream, &request, &self.secret).await?;

                // Obtain reader and writer for inbound and outbound streams
                let (mut client_reader, mut client_writer) = tokio::io::split(inbound_stream);
                let (mut server_reader, mut server_writer) = tokio::io::split(outbound_stream);

                match request.transport_protocol {
                    TransportProtocol::TCP => {
                        tokio::select!(
                            _ = tokio::io::copy(&mut client_reader, &mut server_writer) => (),
                            _ = tokio::io::copy(&mut server_reader, &mut client_writer) => (),
                        );
                    }
                    TransportProtocol::UDP => {
                        let server_reader = TrojanPacketReader::new(server_reader);
                        let server_writer = TrojanPacketWriter::new(server_writer, request);

                        tokio::select!(
                            _ = packet_reader_to_stream_writer(server_reader, &mut client_writer) => (),
                            _ = stream_reader_to_packet_writer(&mut client_reader, server_writer) => (),
                        );
                    }
                }
            }
            SupportedProtocols::SOCKS => {
                return Err(Error::new(ErrorKind::Unsupported, "Unsupported protocol"))
            }
            SupportedProtocols::DIRECT => {
                // StandardStream::new(stream)
                return Err(Error::new(ErrorKind::Unsupported, "Unsupported protocol"));
            }
        };

        info!("Connection finished");
        Ok(())
    }
}
