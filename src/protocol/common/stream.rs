use async_trait::async_trait;
use bytes::BytesMut;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

#[async_trait]
pub trait PacketReader {
    async fn read(&mut self) -> std::io::Result<BytesMut>;
}

#[async_trait]
pub trait PacketWriter {
    async fn write(&mut self, buf: &[u8]) -> std::io::Result<()>;
}

pub enum StandardTcpStream<T> {
    Plain(T),
    RustlsServer(tokio_rustls::server::TlsStream<T>),
    RustlsClient(tokio_rustls::client::TlsStream<T>),
}

impl<S: AsyncRead + AsyncWrite + Unpin + Send> AsyncRead for StandardTcpStream<S> {
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

impl<S: AsyncRead + AsyncWrite + Unpin + Send> AsyncWrite for StandardTcpStream<S> {
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
