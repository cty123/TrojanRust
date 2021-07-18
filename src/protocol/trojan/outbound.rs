use std::task::{Context, Poll};
use std::io::{Result, Error, ErrorKind};
use std::pin::Pin;

use tokio::io::{AsyncRead, AsyncWrite, ReadBuf, BufWriter};

pub struct TrojanOutboundStream<IO> {
    stream: BufWriter<IO>
}

impl<IO> AsyncRead for TrojanOutboundStream<IO>
    where
        IO: AsyncRead + AsyncWrite + Unpin
{
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<Result<()>> {
        return Pin::new(&mut self.stream).poll_read(cx, buf);
    }
}

impl<IO> AsyncWrite for TrojanOutboundStream<IO>
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
        return Pin::new(&mut self.stream).poll_shutdown(cx);
    }
}
