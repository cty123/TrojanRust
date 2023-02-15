use crate::{
    config::base::InboundConfig,
    config::{base::OutboundConfig, tls::make_server_config},
    protocol::{
        common::command::Command,
        trojan::{self, parse},
    },
};

use log::{info, warn};
use quinn::{self, Endpoint};
use std::{io::Result, net::SocketAddr, sync::Arc};
use std::{
    io::{Error, ErrorKind},
    net::ToSocketAddrs,
};
use tokio::net::{TcpStream, UdpSocket};

pub const ALPN_QUIC_HTTP: &[&[u8]] = &[b"hq-29", b"h2", b"h3"];

/// Start running QUIC server
pub async fn start(
    inbound_config: &'static InboundConfig,
    _outboud_config: &'static OutboundConfig,
) -> Result<()> {
    // Extract listening address of the inbound traffic
    let address = (inbound_config.address.clone(), inbound_config.port)
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();

    // Build config for accepting QUIC connection
    let mut tls_config = match &inbound_config.tls {
        Some(tls) => match make_server_config(&tls) {
            Some(cfg) => cfg,
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "Failed to build TLS configuration for QUIC",
                ))
            }
        },
        None => {
            return Err(Error::new(
                ErrorKind::Unsupported,
                "QUIC protocol must have TLS configuration",
            ))
        }
    };
    tls_config.alpn_protocols = ALPN_QUIC_HTTP.iter().map(|&x| x.into()).collect();

    // Build server tls configuration
    let config = quinn::ServerConfig::with_crypto(Arc::new(tls_config));

    // Create QUIC server socket
    let endpoint = Endpoint::server(config, address)?;

    // Start accept loop to handle incomming QUIC connections
    while let Some(conn) = endpoint.accept().await {
        info!("Accepted new QUIC connection");

        tokio::spawn(async move {
            // Establish QUIC connection with handshake
            let connection = match conn.await {
                Ok(c) => c,
                Err(e) => {
                    warn!("Failed to complete QUIC handshake after accepting the initial connection, {}", e);
                    return;
                }
            };

            loop {
                let (mut client_writer, mut client_reader) = match connection.accept_bi().await {
                    Ok((tx, rx)) => (tx, rx),
                    Err(_) => return,
                };

                tokio::spawn(async move {
                    // Read proxy request from the client stream
                    let request = parse(&mut client_reader).await.unwrap().into_request();

                    info!(
                        "Trojan request parsed: ({} {})",
                        request.addr_port.ip.to_string(),
                        request.addr_port.port
                    );

                    // Dispatch connection based on trojan command
                    match request.command {
                        Command::Udp => {
                            let udp_socket = match UdpSocket::bind("0.0.0.0:0").await {
                                Ok(s) => Arc::from(s),
                                Err(e) => {
                                    warn!("Failed to create a local UDP socket: {}", e);
                                    return;
                                }
                            };

                            tokio::select!(
                                _ = trojan::packet::copy_client_reader_to_udp_socket(&mut client_reader, &udp_socket) => (),
                                _ = trojan::packet::copy_udp_socket_to_client_writer(&udp_socket, &mut client_writer, request.addr_port) => ()
                            );
                        }
                        _ => {
                            // Connect to remote server
                            let addr: SocketAddr = match request.addr_port.into() {
                                Ok(addr) => addr,
                                Err(e) => {
                                    warn!("Failed to resolve target dns name: {}", e);
                                    return;
                                }
                            };
                            let outbound_connection = TcpStream::connect(addr).await.unwrap();

                            info!("Established connection");

                            // Transport data between client and remote server
                            let (mut server_reader, mut server_writer) =
                                tokio::io::split(outbound_connection);

                            tokio::select!(
                                _ = tokio::io::copy(&mut client_reader, &mut server_writer) => (),
                                _ = tokio::io::copy(&mut server_reader, &mut client_writer) => ()
                            );
                        }
                    };

                    // Try to gracefully shutdown, nothing we can do if it fails.
                    _ = client_writer.finish().await;
                });
            }
        });
    }

    Ok(())
}
