use std::io::Result;
use std::net::{IpAddr, Ipv4Addr};
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader, ReadBuf};

use crate::protocol::common::addr::IpAddress;
use crate::protocol::common::request::InboundRequest;
use crate::protocol::common::stream::InboundStream;
use crate::protocol::socks5::base::{ServerHello, VERSION};
use crate::protocol::socks5::parser;

pub struct Socks5InboundStream<IO> {
    stream: BufReader<IO>,
}

impl<IO> InboundStream for Socks5InboundStream<IO> where
    IO: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
{
}

impl<IO> AsyncRead for Socks5InboundStream<IO>
where
    IO: AsyncRead + AsyncWrite + Unpin,
{
    #[inline]
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<()>> {
        return Pin::new(&mut self.stream).poll_read(cx, buf);
    }
}

impl<IO> AsyncWrite for Socks5InboundStream<IO>
where
    IO: AsyncRead + AsyncWrite + Unpin,
{
    #[inline]
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize>> {
        return Pin::new(&mut self.stream).poll_write(cx, buf);
    }

    #[inline]
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        return Pin::new(&mut self.stream).poll_flush(cx);
    }

    #[inline]
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        return Pin::new(&mut self.stream).poll_shutdown(cx);
    }
}

/// Simple implementation for SOCKS5 protocol. Only implemented minimum amount of functionality
/// to get TCP working on Firefox browsers.
impl<IO> Socks5InboundStream<IO>
where
    IO: AsyncRead + AsyncWrite + Unpin,
{
    async fn init_ack(mut stream: IO) -> Result<()> {
        let mut buf = [0; 32];

        // Receive the client hello message
        let _ = match stream.read(&mut buf).await {
            Ok(n) => n,
            Err(e) => return Err(e),
        };

        // TODO: Validate client hello message
        // Reply with server hello message
        let server_hello = ServerHello::new(0);
        if let Err(e) = stream.write_all(&server_hello.to_bytes()).await {
            return Err(e);
        }

        Ok(())
    }

    async fn write_request_ack(mut stream: IO, port: u16) -> Result<()> {
        // TODO: Have a better way to write back request ACK
        stream.write_all(&[VERSION, 0, 0, 1]).await?;
        stream
            .write_all(&IpAddress::IpAddr(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))).to_bytes_vec())
            .await?;
        stream.write_u16(port).await?;

        Ok(())
    }
}

impl<IO> Socks5InboundStream<IO>
where
    IO: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
{
    pub async fn new(stream: IO, port: u16) -> Result<(InboundRequest, Box<dyn InboundStream>)> {
        let mut outbound_stream = Socks5InboundStream {
            stream: BufReader::with_capacity(256, stream),
        };

        // Read and reply for the initial client/server hello messages
        Socks5InboundStream::init_ack(&mut outbound_stream).await?;
        let request = parser::parse(&mut outbound_stream).await?;
        Socks5InboundStream::write_request_ack(&mut outbound_stream, port).await?;

        Ok((request.inbound_request(), Box::new(outbound_stream)))
    }
}
