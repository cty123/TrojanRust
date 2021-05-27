use log::{debug};

use std::pin::Pin;
use std::io::{Result, ErrorKind, Error, Write};
use std::task::{Context, Poll};
use std::borrow::Borrow;

use tokio::io::{AsyncRead, AsyncWrite, ReadBuf, AsyncWriteExt, AsyncReadExt};

use crate::protocol::vless::base::VERSION;
use crate::protocol::vless::base::Request;
use std::future::Future;
use futures::FutureExt;

pub struct VlessOutboundStream<IO> {
    stream: IO,
    request: Request,
    is_request_written: bool,
    is_response_read: bool
}

impl<IO> AsyncRead for VlessOutboundStream<IO>
    where
        IO: AsyncRead + AsyncWrite + Unpin
{
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<Result<()>> {
        if !self.is_response_read {
            let mut bytes = [0; 2];
            let mut header = ReadBuf::new(&mut bytes);
            match Pin::new(&mut self.stream).poll_read(cx, &mut header) {
                Poll::Ready(_) => {}
                Poll::Pending => {}
            };
        }
        return Pin::new(&mut self.stream).poll_read(cx, buf);
    }
}

impl<IO> AsyncWrite for VlessOutboundStream<IO>
    where
        IO: AsyncRead + AsyncWrite + Unpin
{
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        if !self.is_request_written {
            let bytes = self.request.to_bytes();
            let len = bytes.len();
            let mut cur = 0;

            while cur < len {
                let n = match Pin::new(&mut self.stream).poll_write(cx, &bytes[cur..len]) {
                    Poll::Ready(res) => match res {
                        Ok(n) => n,
                        Err(e) => return Poll::Ready(Err(e))
                    }
                    Poll::Pending => 0
                };
                cur += n;
            }
            self.is_request_written = true;
        }
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
            is_request_written: true,
            is_response_read: true
        };
    }
}
