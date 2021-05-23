use tokio::io::{AsyncRead, AsyncWrite, ReadBuf, AsyncWriteExt, AsyncReadExt};

use std::pin::Pin;
use std::io::{Result, ErrorKind, Error};
use std::task::{Context, Poll};

use log::{debug};

use crate::protocol::vless::base::VERSION;
use crate::protocol::vless::base::Request;
use std::borrow::Borrow;

pub struct VlessOutboundStream<IO> {
    stream: IO,
    request: Request,
    is_request_written: bool,
    is_response_read: bool,
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
    pub fn new(stream: IO, request: Request) -> VlessOutboundStream<IO> {
        return VlessOutboundStream {
            stream,
            request,
            is_request_written: false,
            is_response_read: false,
        };
    }

    pub async fn write_request(&mut self) -> Result<()> {
        self.stream.write(&self.request.to_bytes()).await?;
        self.is_request_written = true;
        Ok(())
    }

    async fn read_response(&mut self) -> Result<()> {
        return match self.stream.read_u8().await? {
            VERSION => {
                self.is_response_read = true;
                Ok(())
            },
            _ => Err(Error::new(ErrorKind::InvalidInput, "Invalid version number"))
        };
    }
}
