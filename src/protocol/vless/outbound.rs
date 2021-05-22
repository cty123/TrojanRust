use tokio::io::{AsyncRead, AsyncWrite, ReadBuf, AsyncWriteExt, AsyncReadExt};

use std::pin::Pin;
use std::io::Result;
use std::task::{Context, Poll};

use crate::protocol::vless::base::Request;

pub struct VlessOutboundStream<IO> {
    stream: IO,
    is_request_written: bool,
    is_response_read: bool
}

impl<IO> AsyncRead for VlessOutboundStream<IO>
    where
        IO: AsyncRead + AsyncWrite + Unpin
{
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<Result<()>> {
        return Pin::new(&mut self.stream).poll_read(cx, buf);
    }
}

impl<IO> AsyncWrite for VlessOutboundStream<IO>
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

impl<IO> VlessOutboundStream<IO>
    where
        IO: AsyncRead + AsyncWrite + Unpin
{
    pub fn new(stream: IO) -> VlessOutboundStream<IO> {
        return VlessOutboundStream {
            stream,
            is_request_written: false,
            is_response_read: false
        }
    }

    pub async fn write_request(&mut self, request: Request) -> Result<()> {
        self.stream.write(&request.dump_bytes()).await?;
        self.is_request_written = true;
        Ok(())
    }

    pub async fn read_response(&mut self) -> Result<()> {
        let mut buf = [0; 1];
        self.stream.read_exact(&mut buf).await?;
        self.is_response_read = true;
        Ok(())
    }
}