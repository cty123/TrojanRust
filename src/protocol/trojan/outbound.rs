use std::io::Result;
use std::net::IpAddr;
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufStream, ReadBuf};

use crate::protocol::common::addr::IpAddress;
use crate::protocol::common::request::InboundRequest;
use crate::protocol::common::stream::OutboundStream;
use crate::protocol::trojan::base::CRLF;

pub struct TrojanOutboundStream<IO> {
    stream: BufStream<IO>,
}

impl<IO> OutboundStream for TrojanOutboundStream<IO> where
    IO: AsyncRead + AsyncWrite + Unpin + Send + Sync
{
}

impl<IO> AsyncRead for TrojanOutboundStream<IO>
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

impl<IO> AsyncWrite for TrojanOutboundStream<IO>
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

impl<IO> TrojanOutboundStream<IO>
where
    IO: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
{
    pub async fn new(
        stream: IO,
        request: &InboundRequest,
        secret: &[u8],
    ) -> Result<Box<dyn OutboundStream>> {
        let mut stream = TrojanOutboundStream {
            stream: BufStream::with_capacity(256, 256, stream),
        };

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
        Ok(Box::new(stream))
    }
}
