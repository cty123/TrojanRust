pub mod base;
pub mod parser;

use self::base::{ServerHello, VERSION};

use crate::protocol::common::request::InboundRequest;
use crate::protocol::common::stream::{StandardStream, StandardTcpStream};

use std::io::Result;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub async fn accept<T: AsyncRead + AsyncWrite + Unpin>(
    mut stream: StandardTcpStream<T>,
    port: u16,
) -> Result<(InboundRequest, StandardStream<StandardTcpStream<T>>)> {
    // Initialize the handshake process to establish socks connection
    init_ack(&mut stream).await?;

    // Read socks5 request and convert it to universal request
    let request = parser::parse(&mut stream).await?.inbound_request();

    // Write back the request port
    write_request_ack(&mut stream, port).await?;

    Ok((request, StandardStream::new(stream)))
}

async fn init_ack<T: AsyncRead + AsyncWrite + Unpin>(stream: &mut T) -> Result<()> {
    let mut buf = vec![0u8; 32];

    // Receive the client hello message
    stream.read(&mut buf).await?;

    // TODO: Validate client hello message
    // Reply with server hello message
    let server_hello = ServerHello::new(0);
    stream.write_all(&server_hello.to_bytes()).await?;
    stream.flush().await?;

    Ok(())
}

async fn write_request_ack<T: AsyncRead + AsyncWrite + Unpin>(
    mut stream: T,
    port: u16,
) -> Result<()> {
    // TODO: Have a better way to write back request ACK
    stream.write_all(&[VERSION, 0, 0, 1, 127, 0, 0, 1]).await?;
    stream.write_u16(port).await?;
    stream.flush().await?;

    Ok(())
}
