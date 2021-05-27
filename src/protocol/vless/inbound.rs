use log::{debug, error, info, warn};

use std::pin::Pin;
use std::task::{Context, Poll};
use std::io::{Result, Error, ErrorKind};

use tokio::io::{AsyncWrite, AsyncRead, ReadBuf, AsyncReadExt, AsyncWriteExt};

use crate::protocol::vless::base::VERSION;
use crate::protocol::vless::base::{Request, Response};
use crate::protocol::vless::parser::parse;

pub struct VlessInboundStream<IO>
{
    stream: IO,
    is_request_read: bool,
    is_response_written: bool
}

impl<IO> AsyncRead for VlessInboundStream<IO>
    where
        IO: AsyncRead + AsyncWrite + Unpin
{
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<Result<()>> {
        return Pin::new(&mut self.stream).poll_read(cx, buf);
    }
}

impl<IO> AsyncWrite for VlessInboundStream<IO>
    where
        IO: AsyncRead + AsyncWrite + Unpin
{
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        if !self.is_response_written {
            let bytes = [1; 0];
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
            self.is_response_written = true;
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

impl<IO> VlessInboundStream<IO>
    where
        IO: AsyncRead + AsyncWrite + Unpin
{
    pub fn new(stream: IO) -> VlessInboundStream<IO> {
        return VlessInboundStream {
            stream,
            is_request_read: false,
            is_response_written: false
        }
    }

    pub async fn read_request(&mut self) -> Result<Request> {
        let request = match parse(&mut self.stream).await {
            Ok(r) => r,
            Err(e) => return Err(Error::new(ErrorKind::InvalidInput, e))
        };
        debug!("Read request {}", request.request_addr_port());
        self.is_request_read = true;
        Ok(request)
    }

    pub async fn ack_request(&mut self) -> Result<()>{
        let response = Response::new(VERSION);
        self.is_response_written = true;
        return self.stream.write_all(&response.to_bytes()).await;
    }
}
