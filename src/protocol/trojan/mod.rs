mod base;
mod handler;
mod parser;

pub mod packet;

pub use self::base::CRLF;
pub use self::base::HEX_SIZE;
pub use self::handler::{handle_client_data, handle_server_data};
pub use self::parser::parse;
pub use self::parser::parse_udp;

use crate::protocol::common::addr::IpAddress;
use crate::protocol::common::{request::InboundRequest, stream::StandardTcpStream};

use std::io::{Error, ErrorKind, Result};
use std::net::IpAddr;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

/// Helper function to accept an abstract TCP stream to Trojan connection
pub async fn accept<T: AsyncRead + AsyncWrite + Unpin + Send>(
    mut stream: StandardTcpStream<T>,
    secret: &[u8],
) -> Result<(InboundRequest, StandardTcpStream<T>)> {
    // Read trojan request header and generate request header
    let request = parse(&mut stream).await?;

    // Validate the request secret and decide if the connection should be accepted
    if !request.validate(secret) {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Received invalid hex value",
        ));
    }

    Ok((request.inbound_request(), stream))
}

/// Helper function to establish Trojan connection to remote server
pub async fn handshake<T: AsyncWrite + Unpin>(
    mut stream: T,
    request: &InboundRequest,
    secret: &[u8],
) -> Result<T> {
    // Write request header
    stream.write_all(secret).await?;
    stream.write_u16(CRLF).await?;
    stream.write_u8(request.command.to_byte()).await?;
    stream.write_u8(request.atype.to_byte()).await?;
    match &request.addr {
        IpAddress::IpAddr(IpAddr::V4(ipv4)) => {
            stream.write_all(&ipv4.octets()).await?;
        }
        IpAddress::IpAddr(IpAddr::V6(ipv6)) => {
            stream.write_all(&ipv6.octets()).await?;
        }
        IpAddress::Domain(domain) => {
            stream.write_all(&domain.to_bytes()).await?;
        }
    }
    stream.write_u16(request.port).await?;
    stream.write_u16(CRLF).await?;
    stream.flush().await?;

    // Return the outbound stream itself
    Ok(stream)
}
