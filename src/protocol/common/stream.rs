use async_trait::async_trait;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

#[async_trait]
pub trait IOStream: AsyncRead + AsyncWrite + Unpin + Send + Sync {}

#[async_trait]
pub trait InboundStream: AsyncRead + AsyncWrite + Unpin + Send + Sync {}

#[async_trait]
pub trait OutboundStream: AsyncRead + AsyncWrite + Unpin + Send + Sync {}

#[async_trait]
pub trait PacketStream: AsyncRead + AsyncWrite + Unpin + Send + Sync {}

pub enum StandardTcpStream<T> {
    Plain(T),
    RustlsServer(tokio_rustls::server::TlsStream<T>),
    RustlsClient(tokio_rustls::client::TlsStream<T>),
}

impl<S: AsyncRead + AsyncWrite + Unpin> AsyncRead for StandardTcpStream<S> {
    #[inline]
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            StandardTcpStream::Plain(ref mut s) => Pin::new(s).poll_read(cx, buf),
            StandardTcpStream::RustlsServer(s) => Pin::new(s).poll_read(cx, buf),
            StandardTcpStream::RustlsClient(s) => Pin::new(s).poll_read(cx, buf),
        }
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> AsyncWrite for StandardTcpStream<S> {
    #[inline]
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        match self.get_mut() {
            StandardTcpStream::Plain(ref mut s) => Pin::new(s).poll_write(cx, buf),
            StandardTcpStream::RustlsServer(ref mut s) => Pin::new(s).poll_write(cx, buf),
            StandardTcpStream::RustlsClient(ref mut s) => Pin::new(s).poll_write(cx, buf),
        }
    }

    #[inline]
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            StandardTcpStream::Plain(ref mut s) => Pin::new(s).poll_flush(cx),
            StandardTcpStream::RustlsServer(ref mut s) => Pin::new(s).poll_flush(cx),
            StandardTcpStream::RustlsClient(ref mut s) => Pin::new(s).poll_flush(cx),
        }
    }

    #[inline]
    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            StandardTcpStream::Plain(ref mut s) => Pin::new(s).poll_shutdown(cx),
            StandardTcpStream::RustlsServer(ref mut s) => Pin::new(s).poll_shutdown(cx),
            StandardTcpStream::RustlsClient(ref mut s) => Pin::new(s).poll_shutdown(cx),
        }
    }
}

pub struct StandardStream<T> {
    stream: T,
}

impl<T> StandardStream<T> {
    pub fn new(stream: T) -> Self {
        Self { stream }
    }

    pub fn into_inner(self) -> T {
        self.stream
    }
}
