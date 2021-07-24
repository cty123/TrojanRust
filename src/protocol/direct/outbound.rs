use async_trait::async_trait;

use std::io::Result;
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::{AsyncRead, AsyncWrite, BufReader, ReadBuf};

use crate::protocol::common::stream::OutboundStream;

pub struct DirectOutboundStream<IO>
where
    IO: AsyncRead + AsyncWrite + Unpin,
{
    stream: BufReader<IO>,
}

#[async_trait]
impl<IO> OutboundStream for DirectOutboundStream<IO> where
    IO: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
{
}

impl<IO> DirectOutboundStream<IO>
where
    IO: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
{
    pub fn new(stream: IO) -> Box<dyn OutboundStream> {
        return Box::new(DirectOutboundStream {
            stream: BufReader::with_capacity(1024, stream),
        });
    }
}

impl<IO> AsyncRead for DirectOutboundStream<IO>
where
    IO: AsyncRead + AsyncWrite + Unpin,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<()>> {
        return Pin::new(&mut self.stream).poll_read(cx, buf);
    }
}

impl<IO> AsyncWrite for DirectOutboundStream<IO>
where
    IO: AsyncRead + AsyncWrite + Unpin,
{
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize>> {
        return Pin::new(&mut self.stream).poll_write(cx, buf);
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        return Pin::new(&mut self.stream).poll_flush(cx);
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        return Pin::new(&mut self.stream).poll_shutdown(cx);
    }
}
