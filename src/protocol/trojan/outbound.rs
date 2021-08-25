use std::io::Result;
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::{AsyncRead, AsyncWrite, BufReader, ReadBuf};

use crate::protocol::common::request::InboundRequest;
use crate::protocol::common::stream::OutboundStream;
use crate::protocol::trojan::base::Request;

pub struct TrojanOutboundStream<IO> {
    stream: BufReader<IO>,
    request: Request,
    header_written: bool,
}

impl<IO> OutboundStream for TrojanOutboundStream<IO> where
    IO: AsyncRead + AsyncWrite + Unpin + Send + Sync
{
}

impl<IO> AsyncRead for TrojanOutboundStream<IO>
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

impl<IO> AsyncWrite for TrojanOutboundStream<IO>
where
    IO: AsyncRead + AsyncWrite + Unpin,
{
    #[inline]
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize>> {
        if !self.header_written {
            let mut pos = 0;
            let header = self.request.to_bytes();
            while pos < header.len() {
                match Pin::new(&mut self.stream).poll_write(cx, &header) {
                    Poll::Ready(res) => match res {
                        Ok(n) => pos += n,
                        Err(e) => return Poll::Ready(Err(e)),
                    },
                    Poll::Pending => return Poll::Pending,
                }
            }
            self.header_written = true;
        }

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

impl<IO> TrojanOutboundStream<IO>
where
    IO: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
{
    pub fn new(
        stream: IO,
        request: &InboundRequest,
        secret: [u8; 56],
    ) -> Box<dyn OutboundStream> {
        Box::new(TrojanOutboundStream {
            stream: BufReader::with_capacity(256, stream),
            request: Request::from_request(request, secret),
            header_written: false,
        })
    }
}
