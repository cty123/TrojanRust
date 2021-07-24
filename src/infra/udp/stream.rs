/**
 * Have to implement this unfortunately, since tokio doesn't have UdpStream out of box
 * This implementation should be removed if tokio or extension of tokio packages offers
 * UdpStream.
 */

use log::info;

use std::io::{Result, Error};
use std::convert::TryInto;
use std::task::{Context, Poll};
use std::pin::Pin;

use tokio::net::{UdpSocket, ToSocketAddrs};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pub struct UdpStream {
    io: UdpSocket,
}

impl UdpStream {
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> Result<UdpStream> {
        let udp_socket = UdpSocket::bind("0.0.0.0:0").await?;
        udp_socket.connect(addr).await?;
        return Ok(UdpStream {
            io: udp_socket
        });
    }
}

impl AsyncRead for UdpStream {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<Result<()>> {
        return Pin::new(&mut self.io).poll_recv(cx, buf);
    }
}

impl AsyncWrite for UdpStream {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        info!("Write bytes {:?}", buf);
        return Pin::new(&mut self.io).poll_send(cx, buf);
    }

    #[inline]
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        return Poll::Ready(Ok(()));
    }

    #[inline]
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        return Poll::Ready(Ok(()));
    }
}