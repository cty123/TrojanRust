use crate::protocol::common::addr::IpAddress;
use crate::protocol::common::request::InboundRequest;
use crate::protocol::trojan::parser::parse_udp;
use std::io::{Error, ErrorKind, Result};
use std::net::IpAddr;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::UdpSocket;

pub async fn handle_client_data<R: AsyncRead + Unpin>(
    client_reader: &mut R,
    server_writer: &UdpSocket,
) -> Result<()> {
    loop {
        // Parse the UDP header and obtain the payload size
        let payload_length = parse_udp(client_reader).await?;

        // Create payload buffer
        let mut payload = vec![0u8; payload_length];

        // Need to double check if the buffer is really filled
        match client_reader.read_exact(&mut payload).await {
            Ok(n) if n == payload_length as usize => (),
            Ok(_) => {
                return Err(Error::new(
                    ErrorKind::ConnectionReset,
                    "incorrect length of payload read",
                ))
            }
            Err(e) => return Err(e),
        }

        // Forward the data back to the client
        server_writer.send(&payload).await?;
    }
}

pub async fn handle_server_data<W: AsyncWrite + Unpin>(
    client_writer: &mut W,
    server_reader: &UdpSocket,
    request: InboundRequest,
) -> Result<()> {
    loop {
        let mut buf = vec![0; 4096];

        // Receive data from the reader
        let size = server_reader.recv(&mut buf).await?;

        // Write back the address type of the trojan request
        client_writer.write_u8(request.atype.to_byte()).await?;

        // Write back the address of the trojan request
        match request.addr {
            IpAddress::IpAddr(IpAddr::V4(addr)) => {
                client_writer.write_all(&addr.octets()).await?;
            }
            IpAddress::IpAddr(IpAddr::V6(addr)) => {
                client_writer.write_all(&addr.octets()).await?;
            }
            IpAddress::Domain(ref domain) => {
                client_writer
                    .write_u8(domain.to_bytes().len() as u8)
                    .await?;
                client_writer.write_all(domain.to_bytes()).await?;
            }
        }

        // Write port, payload size, CRLF, and the payload data into the stream
        client_writer.write_u16(request.port).await?;
        client_writer.write_u16(size as u16).await?;
        client_writer.write_u16(0x0D0A).await?;
        client_writer.write_all(&mut buf[..size]).await?;

        // Make sure to flush the writer as we may have buffered writer implemented for the client connection
        client_writer.flush().await?;
    }
}
