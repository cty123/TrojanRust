use std::io::Result;
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::{AsyncRead, AsyncWrite, BufReader, ReadBuf};

use async_trait::async_trait;
use log::info;

use crate::protocol::common::stream::InboundStream;
use crate::protocol::trojan::parser::parse;

pub struct TrojanInboundStream<IO> {
    stream: BufReader<IO>,
}

impl<IO> AsyncRead for TrojanInboundStream<IO>
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

impl<IO> AsyncWrite for TrojanInboundStream<IO>
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

#[async_trait]
impl<IO> InboundStream for TrojanInboundStream<IO>
where
    IO: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
{
    async fn handshake(&mut self) -> Result<String> {
        let request = parse(&mut self.stream).await?;
        info!("Received trojan request: {}", request.dump_request());
        return Ok(request.request_addr_port());
    }
}

impl<IO> TrojanInboundStream<IO>
where
    IO: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
{
    pub fn new(stream: IO) -> Box<dyn InboundStream> {
        return Box::new(TrojanInboundStream {
            stream: tokio::io::BufReader::with_capacity(2048, stream),
        });
    }
}
