use bytes::BufMut;
use log::info;
use once_cell::sync::OnceCell;
use std::io::{Error, ErrorKind};
use std::net::IpAddr;
use std::sync::Arc;
use std::{io, net::SocketAddr};
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::net::{TcpStream, UdpSocket};
use tokio::sync::mpsc::Sender;
use tonic::Status;

use crate::protocol::common::addr::IpAddress;
use crate::protocol::trojan;
use crate::protocol::trojan::packet::{TrojanPacketReader};
use crate::{
    protocol::common::request::InboundRequest,
    proxy::base::SupportedProtocols,
    transport::{grpc_stream::GrpcDataStream, grpc_transport::Hunk},
};

/// Static life time TCP server outbound traffic handler to avoid ARC
/// The handler is initialized through init() function
static GRPC_HANDLER: OnceCell<GrpcHandler> = OnceCell::new();

/// GrpcHandler is responsible for handling outbound traffic for GRPC inbound streams
pub struct GrpcHandler {
    destination: Option<SocketAddr>,
    protocol: SupportedProtocols,
}

impl GrpcHandler {
    pub fn new() -> &'static GrpcHandler {
        GRPC_HANDLER.get_or_init(|| Self {
            destination: None,
            protocol: SupportedProtocols::TROJAN,
        })
    }

    pub async fn handle_hunk(
        &self,
        mut client_reader: GrpcDataStream<Hunk>,
        client_writer: Sender<Result<Hunk, Status>>,
        request: InboundRequest,
    ) -> io::Result<()> {
        return match request.command {
            crate::protocol::common::command::Command::Connect => {
                // Establish connection to remote server as specified by proxy request
                let (mut server_reader, mut server_writer) =
                    match TcpStream::connect((request.addr.to_string(), request.port as u16)).await
                    {
                        Ok(stream) => tokio::io::split(stream),
                        Err(e) => {
                            // error!("Failed to connect to {}:{}", address, port);
                            return Err(e);
                        }
                    };

                // Spawn two concurrent coroutines to transport the data between client and server
                tokio::select!(
                    _ = tokio::io::copy(&mut client_reader, &mut server_writer) => (),
                    _ = write_back_tcp_traffic(&mut server_reader, client_writer) => (),
                );

                Ok(())
            }
            crate::protocol::common::command::Command::Udp => {
                // Establish UDP connection to remote host
                let socket = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
                match socket
                    .connect((request.addr.to_string(), request.port as u16))
                    .await
                {
                    Ok(_) => (),
                    Err(e) => {
                        return Err(Error::new(
                            ErrorKind::ConnectionRefused,
                            format!("failed to connect to udp {}: {}", request.addr, e),
                        ))
                    }
                }

                // Setup the reader and writer for both the client and server so that we can transport the data
                let (server_reader, server_writer) = (socket.clone(), socket.clone());
                let client_packet_reader = TrojanPacketReader::new(client_reader);

                tokio::select!(
                    _ = trojan::packet::packet_reader_to_udp_packet_writer(client_packet_reader, server_writer) => (),
                    _ = write_back_udp_traffic(client_writer, server_reader, request) => ()
                );

                Ok(())
            }
            crate::protocol::common::command::Command::Bind => Err(Error::new(
                ErrorKind::Unsupported,
                "Bind command is not supported in Trojan",
            )),
        };
    }
}

async fn write_back_tcp_traffic<R: AsyncRead + Unpin>(
    mut reader: R,
    writer: Sender<Result<Hunk, Status>>,
) -> io::Result<()> {
    loop {
        let mut buf = Vec::with_capacity(4096);

        match reader.read_buf(&mut buf).await {
            Ok(_) => (),
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "Failed to read data from remote server",
                ))
            }
        }

        match writer.send(Ok(Hunk { data: buf })).await {
            Ok(_) => (),
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "Failed to send data to client",
                ))
            }
        }
    }
}

async fn write_back_udp_traffic(
    client_sender: Sender<Result<Hunk, Status>>,
    udp_socket: Arc<UdpSocket>,
    request: InboundRequest,
) -> io::Result<()> {
    loop {
        let mut data = vec![0u8; 4096];
        let n = match udp_socket.recv(&mut data).await {
            Ok(n) => n,
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "Failed to read data from remote server",
                ))
            }
        };

        let mut buf = Vec::with_capacity(4096);

        // Write address type to remote
        buf.put_u8(request.atype as u8);

        // Write back the address of the trojan request
        match request.addr {
            IpAddress::IpAddr(IpAddr::V4(addr)) => {
                buf.put_slice(&addr.octets());
            }
            IpAddress::IpAddr(IpAddr::V6(addr)) => {
                buf.put_slice(&addr.octets());
            }
            IpAddress::Domain(ref domain) => {
                buf.put_u8(domain.to_bytes().len() as u8);
                buf.put_slice(domain.to_bytes());
            }
        }

        // Write port, payload size, CRLF, and the payload data into the stream
        buf.put_u16(request.port);
        buf.put_u16(n as u16);
        buf.put_u16(0x0D0A);
        buf.put_slice(&data[..n]);

        match client_sender.send(Ok(Hunk { data: buf })).await {
            Ok(_) => (),
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "Failed to send data to client",
                ))
            }
        }
    }
}
