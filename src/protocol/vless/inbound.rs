use log::{debug, error, info, warn};
use tokio::io::{AsyncWrite, AsyncRead, ReadBuf, AsyncReadExt, AsyncWriteExt};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::io::{Result, Error, ErrorKind};

use crate::protocol::vless::base::Request;
use crate::protocol::vless::parser::parse;

pub struct VlessInboundStream<IO>
{
    stream: IO,
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
            stream
        }
    }

    pub async fn read_request(&mut self) -> Result<Request> {
        let request = match parse(&mut self.stream).await {
            Ok(r) => r,
            Err(e) => return Err(Error::new(ErrorKind::InvalidInput, e))
        };

        Ok(request)
    }

    pub async fn ack_request(&mut self) -> Result<()>{
        let ack = [1 as u8];
        return self.stream.write_all(&ack).await;
    }
}

// impl<IO> AsyncReadExt for VlessInboundStream<IO>
//     where
//         IO: AsyncRead + AsyncWrite + Unpin
// {
//
// }
//
// impl<IO> AsyncWriteExt for VlessInboundStream<IO>
//     where
//         IO: AsyncRead + AsyncWrite + Unpin
// {
//
// }