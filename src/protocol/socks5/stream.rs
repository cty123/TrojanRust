use log::{debug, error, info, warn};

use std::task::{Context, Poll};
use std::pin::Pin;
use std::io::Result;
use std::io::Error;
use std::io::ErrorKind;

use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt, WriteHalf, ReadHalf, Split, ReadBuf};
use futures::{future, StreamExt, SinkExt};

use crate::protocol::socks5::base::{Request, ServerHello, RequestAck};
use crate::protocol::socks5::parser;

pub struct Socks5Stream<IO> {
    stream: IO,
    port: [u8; 2],
}

impl<IO> Socks5Stream<IO>
    where
        IO: AsyncRead + AsyncWrite + Unpin
{
    pub fn new(stream: IO, port: [u8; 2]) -> Socks5Stream<IO> {
        Socks5Stream {
            stream,
            port,
        }
    }
}

impl<IO> AsyncRead for Socks5Stream<IO>
    where
        IO: AsyncRead + AsyncWrite + Unpin
{
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<Result<()>> {
        return Pin::new(&mut self.stream).poll_read(cx, buf);
    }
}

impl<IO> AsyncWrite for Socks5Stream<IO>
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

impl<IO> Socks5Stream<IO>
    where
        IO: AsyncRead + AsyncWrite + Unpin
{
    pub async fn handshake(&mut self) -> Result<Request> {
        // Read and reply for the initial client/server hello messages
        if let Err(e) = self.init_ack().await {
            return Err(e);
        };

        // Read request and parse it
        let request = match self.read_request().await {
            Ok(r) => r,
            Err(e) => return Err(e)
        };

        info!("Received socks5 request: {}", request.dump_request());

        // Reply ACK message for request message
        if let Err(e) = self.write_request_ack().await {
            return Err(e);
        }

        Ok(request)
    }

    async fn init_ack(&mut self) -> Result<()> {
        let mut buf = [0; 32];

        // Receive the client hello message
        let n = match self.stream.read(&mut buf).await {
            Ok(n) => n,
            Err(e) => return Err(e)
        };

        debug!("Read {} bytes of data: {:?}", n, &buf[0..n]);

        // TODO: Validate client hello message
        // Reply with server hello message
        let server_hello = ServerHello::new(5, 0);
        if let Err(e) = self.stream.write_all(&server_hello.to_bytes()).await {
            return Err(e);
        }

        debug!("Wrote {} bytes of data: {:?}", 2, server_hello.to_bytes());

        Ok(())
    }

    async fn read_request(&mut self) -> Result<Request> {
        let request = parser::parse(&mut self.stream).await?;
        return Ok(request)
    }

    async fn write_request_ack(&mut self)-> Result<()> {
        // TODO: Have a better way to write back request ACK
        let ack = RequestAck::new(5, 0, 0, 1, [127, 0, 0, 1], self.port);
        if let Err(e) = self.stream.write_all(&ack.to_bytes()).await {
            error!("Failed to write to socket, err = {:?}", e);
            return Err(e);
        }

        debug!("Reply request ACK {:?}", ack.to_bytes());

        Ok(())
    }
}