use std::io::{Error, ErrorKind, Result};
use std::pin::Pin;
use std::task::{Context, Poll};

use log::info;
use tokio::io::{AsyncRead, AsyncWrite, BufStream, ReadBuf};

use crate::protocol::common::request::InboundRequest;
use crate::protocol::common::stream::InboundStream;
use crate::protocol::trojan::parser::parse;

pub struct TrojanInboundStream<IO> {
    stream: BufStream<IO>,
}

impl<IO> InboundStream for TrojanInboundStream<IO> where
    IO: AsyncRead + AsyncWrite + Unpin + Send + Sync
{
}

impl<IO> AsyncRead for TrojanInboundStream<IO>
where
    IO: AsyncRead + AsyncWrite + Unpin,
{
    #[inline]
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<()>> {
        return Pin::new(&mut self.stream).poll_read(cx, buf);
    }
}

impl<IO> AsyncWrite for TrojanInboundStream<IO>
where
    IO: AsyncRead + AsyncWrite + Unpin,
{
    #[inline]
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize>> {
        return Pin::new(&mut self.stream).poll_write(cx, buf);
    }

    #[inline]
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        return Pin::new(&mut self.stream).poll_flush(cx);
    }

    #[inline]
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        return Pin::new(&mut self.stream).poll_shutdown(cx);
    }
}

impl<IO> TrojanInboundStream<IO>
where
    IO: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
{
    pub async fn new(
        stream: IO,
        secret: &[u8],
    ) -> Result<(InboundRequest, Box<dyn InboundStream>)> {
        let mut outbound_stream = TrojanInboundStream {
            stream: BufStream::with_capacity(256, 256, stream),
        };

        let request = parse(&mut outbound_stream).await?;

        info!("Received request: trojan {}", request.to_string());

        if !request.validate(secret) {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Received invalid hex value",
            ));
        }

        Ok((request.inbound_request(), Box::new(outbound_stream)))
    }
}
