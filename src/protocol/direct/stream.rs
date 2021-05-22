use tokio::io::{AsyncWrite, AsyncRead, ReadBuf};
use tokio::net::TcpStream;
use tokio_rustls::{TlsConnector};
use tokio_rustls::client::{TlsStream};

use std::io::{Result, Error};
use tokio_rustls::rustls::ClientConfig;
use std::sync::Arc;
use tokio_rustls::webpki::DNSNameRef;
use std::task::{Context, Poll};
use std::pin::Pin;

pub struct DirectStream<IO>
    where
        IO: AsyncRead + AsyncWrite + Unpin
{
    stream: IO,
    use_tls: bool,
}

impl<IO> AsyncRead for DirectStream<IO>
    where
        IO: AsyncRead + AsyncWrite + Unpin
{
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<Result<()>> {
        return Pin::new(&mut self.stream).poll_read(cx, buf);
    }
}

impl<IO> AsyncWrite for DirectStream<IO>
    where
        IO: AsyncRead + AsyncWrite + Unpin
{
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        return Pin::new(&mut self.stream).poll_write(cx, buf);
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        return Pin::new(&mut self.stream).poll_flush(cx);
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        return Pin::new(&mut self.stream).poll_flush(cx);
    }
}

impl<IO> DirectStream<IO>
    where
        IO: AsyncRead + AsyncWrite + Unpin
{
    pub fn new(stream: IO, use_tls: bool) -> DirectStream<IO> {
        return DirectStream {
            stream,
            use_tls,
        };
    }
}
