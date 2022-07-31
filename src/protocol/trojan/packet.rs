use crate::protocol::common::addr::{IpAddrPort, IpAddress};
use crate::protocol::common::atype::Atype;
use crate::protocol::common::request::InboundRequest;
use crate::protocol::trojan::base::CRLF;
use crate::protocol::trojan::parser::parse_udp;
use crate::transport::grpc_stream::GrpcDataReaderStream;
use crate::transport::grpc_transport::Hunk;

use log::debug;
use std::io::{self, Cursor, Error, ErrorKind};
use std::net::{IpAddr, SocketAddr};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;
use tonic::Streaming;

/// Define the size of the buffer used to transport the data back and forth
const BUF_SIZE: usize = 4096;

/// According the official documentation for Trojan protocol, the UDP data will be segmented into Trojan UDP packets,
/// which allows the outbound handler to also forward them as real UDP packets to the desired destinations.
/// Link: https://trojan-gfw.github.io/trojan/protocol.html
pub struct TrojanUdpPacketHeader {
    pub atype: Atype,
    pub dest: SocketAddr,
    pub payload_size: usize,
}

pub async fn copy_client_reader_to_udp_socket<R: AsyncRead + Unpin>(
    mut client_reader: R,
    server_writer: &UdpSocket,
) -> io::Result<()> {
    let mut read_buf = vec![0u8; BUF_SIZE];

    loop {
        let header = parse_udp(&mut client_reader).await?;

        debug!(
            "Forwarding {} bytes to {}",
            header.payload_size, header.dest
        );

        assert!(
            header.payload_size <= BUF_SIZE,
            "Payload size exceeds read buffer size"
        );

        let size = client_reader
            .read_exact(&mut read_buf[..header.payload_size])
            .await?;

        assert!(
            size == header.payload_size,
            "Failed to read the entire trojan udp packet, expect: {} bytes, read: {} bytes",
            header.payload_size,
            size
        );

        server_writer
            .send_to(&read_buf[..header.payload_size], header.dest)
            .await?;
    }
}

pub async fn copy_udp_socket_to_client_writer<W: AsyncWrite + Unpin>(
    server_reader: &UdpSocket,
    mut client_writer: W,
    addr: IpAddrPort,
) -> io::Result<()> {
    let mut read_buf = vec![0u8; BUF_SIZE];
    let (addr, port) = (addr.ip, addr.port);

    loop {
        let (size, _dest) = server_reader.recv_from(&mut read_buf).await?;

        match addr {
            IpAddress::IpAddr(IpAddr::V4(addr)) => {
                client_writer.write_u8(Atype::IPv4 as u8).await?;
                client_writer.write_all(&addr.octets()).await?;
            }
            IpAddress::IpAddr(IpAddr::V6(addr)) => {
                client_writer.write_u8(Atype::IPv6 as u8).await?;
                client_writer.write_all(&addr.octets()).await?;
            }
            IpAddress::Domain(ref domain) => {
                client_writer.write_u8(Atype::DomainName as u8).await?;
                client_writer
                    .write_u8(domain.as_bytes().len() as u8)
                    .await?;
                client_writer.write_all(domain.as_bytes()).await?;
            }
        }

        client_writer.write_u16(port).await?;
        client_writer.write_u16(size as u16).await?;
        client_writer.write_u16(CRLF).await?;
        client_writer.write_all(&read_buf[..size]).await?;
        client_writer.flush().await?;
    }
}

pub async fn copy_client_reader_to_udp_server_writer<
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
>(
    mut client_reader: R,
    mut server_writer: W,
    request: InboundRequest,
) -> io::Result<()> {
    let mut read_buf = vec![0u8; BUF_SIZE];

    loop {
        let size = client_reader.read(&mut read_buf).await?;

        server_writer.write_u8(request.atype as u8).await?;

        match request.addr_port.ip {
            IpAddress::IpAddr(IpAddr::V4(addr)) => {
                server_writer.write_all(&addr.octets()).await?;
            }
            IpAddress::IpAddr(IpAddr::V6(addr)) => {
                server_writer.write_all(&addr.octets()).await?;
            }
            IpAddress::Domain(ref domain) => {
                server_writer
                    .write_u8(domain.as_bytes().len() as u8)
                    .await?;
                server_writer.write_all(domain.as_bytes()).await?;
            }
        }

        server_writer.write_u16(request.addr_port.port).await?;
        server_writer.write_u16(size as u16).await?;
        server_writer.write_u16(CRLF).await?;
        server_writer.write_all(&read_buf[..size]).await?;
        server_writer.flush().await?;
    }
}

pub async fn copy_udp_server_reader_to_client_writer<
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
>(
    mut server_reader: R,
    mut client_writer: W,
) -> io::Result<()> {
    let mut read_buf = vec![0u8; BUF_SIZE];

    loop {
        let header = parse_udp(&mut server_reader).await?;

        server_reader
            .read_exact(&mut read_buf[..header.payload_size])
            .await?;

        client_writer
            .write_all(&read_buf[..header.payload_size])
            .await?;
    }
}

pub async fn copy_client_reader_to_server_grpc_writer<R: AsyncRead + Unpin>(
    mut client_reader: R,
    server_writer: Sender<Hunk>,
    request: InboundRequest,
) -> io::Result<()> {
    loop {
        let mut read_buf = vec![0u8; BUF_SIZE];

        let n = client_reader.read(&mut read_buf).await?;

        read_buf.truncate(n);

        let mut cursor = Cursor::new(vec![0u8; 512]);

        cursor.write_u8(request.atype as u8).await?;

        match request.addr_port.ip {
            IpAddress::IpAddr(IpAddr::V4(addr)) => {
                cursor.write_all(&addr.octets()).await?;
            }
            IpAddress::IpAddr(IpAddr::V6(addr)) => {
                cursor.write_all(&addr.octets()).await?;
            }
            IpAddress::Domain(ref domain) => {
                cursor.write_u8(domain.as_bytes().len() as u8).await?;
                cursor.write_all(domain.as_bytes()).await?;
            }
        }

        cursor.write_u16(request.addr_port.port).await?;
        cursor.write_u16(n as u16).await?;
        cursor.write_u16(CRLF).await?;

        let (pos, mut header) = (cursor.position(), cursor.into_inner());
        header.truncate(pos as usize);

        if let Err(_) = server_writer.send(Hunk { data: header }).await {
            return Err(Error::new(
                ErrorKind::ConnectionReset,
                "Failed to send GRPC packet",
            ));
        }

        if let Err(_) = server_writer.send(Hunk { data: read_buf }).await {
            return Err(Error::new(
                ErrorKind::ConnectionReset,
                "Failed to send GRPC packet",
            ));
        }
    }
}

pub async fn copy_server_grpc_reader_to_client_writer<W: AsyncWrite + Unpin>(
    server_reader: Streaming<Hunk>,
    mut client_writer: W,
) -> io::Result<()> {
    let mut server_reader = GrpcDataReaderStream::from_reader(server_reader);
    let mut read_buf = vec![0u8; BUF_SIZE];

    loop {
        let header = parse_udp(&mut server_reader).await?;

        server_reader
            .read_exact(&mut read_buf[..header.payload_size])
            .await?;

        client_writer
            .write_all(&read_buf[..header.payload_size])
            .await?;
    }
}
