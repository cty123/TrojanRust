use std::borrow::Borrow;
use std::cmp::min;
use std::io::{Error, ErrorKind, Result};
use std::pin::Pin;
use std::task::{Context, Poll};

use async_trait::async_trait;
use bytes::{Buf, BufMut, BytesMut};
use log::{info, warn};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::UdpSocket;

use crate::protocol::common::addr::{IpAddress, IPV4_SIZE, IPV6_SIZE};
use crate::protocol::common::atype::Atype;
use crate::protocol::common::stream::OutboundStream;

const BYTES_ATYPE: usize = 1;
const BYTES_ADDR_SIZE: usize = 1;
const BYTES_PORT: usize = 2;
const BYTES_PAYLOAD_SIZE: usize = 2;
const BYTES_CRLF: usize = 2;

enum State {
    Atype,
    AddrSize,
    Addr,
    Port,
    PayloadSize,
    CRLF,
    Payload,
}

pub struct PacketTrojanOutboundStream {
    udp_socket: UdpSocket,

    // Internal states
    buffer: BytesMut,
    state: State,

    // UDP request packet info
    atype: Atype,
    addr: IpAddress,
    addr_size: usize,
    port: u16,
    payload_size: usize,
}

#[async_trait]
impl OutboundStream for PacketTrojanOutboundStream {}

impl AsyncRead for PacketTrojanOutboundStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<()>> {
        buf.put_slice(&[self.atype.to_byte()]);
        buf.put_slice(&self.addr.to_bytes_vec());
        buf.put_slice(&self.port.to_be_bytes());
        buf.put_slice(&[0, 0, 0x0D, 0x0A]);
        let header_len = buf.filled().len();

        return match self.udp_socket.poll_recv_from(cx, buf) {
            Poll::Ready(res) => match res {
                Ok(_) => {
                    let payload_len = (buf.filled().len() - header_len) as u16;
                    let len_bytes = payload_len.to_be_bytes();
                    buf.filled_mut()[header_len - 4] = len_bytes[0];
                    buf.filled_mut()[header_len - 3] = len_bytes[1];
                    Poll::Ready(Ok(()))
                }
                Err(e) => {
                    info!("Failed to read from udp, {}", e);
                    Poll::Ready(Err(e))
                }
            },
            Poll::Pending => Poll::Pending,
        };
    }
}

impl AsyncWrite for PacketTrojanOutboundStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize>> {
        let mut pos: usize = 0;

        while pos < buf.len() {
            match &self.state {
                State::Atype => {
                    self.atype = match Atype::from(buf[pos]) {
                        Ok(atype) => atype,
                        Err(e) => return Poll::Ready(Err(e)),
                    };
                    self.state = State::AddrSize;
                    pos += 1;
                }
                State::AddrSize => {
                    self.addr_size = match self.atype {
                        Atype::IPv4 => IPV4_SIZE,
                        Atype::IPv6 => IPV6_SIZE,
                        Atype::DomainName => {
                            pos += 1;
                            usize::from(buf[pos])
                        }
                    };
                    self.state = State::Addr;
                }
                State::Addr => {
                    let addr_size = self.addr_size;
                    pos = self.read_field(pos, addr_size, buf);

                    if self.buffer.len() >= self.addr_size {
                        self.addr = match self.atype {
                            Atype::IPv4 => IpAddress::from_u32(self.buffer.get_u32()),
                            Atype::IPv6 => IpAddress::from_u128(self.buffer.get_u128()),
                            Atype::DomainName => IpAddress::from_vec(self.buffer.to_vec()),
                        };
                        self.buffer.clear();
                        self.state = State::Port;
                    }
                }
                State::Port => {
                    pos = self.read_field(pos, BYTES_PORT, buf);

                    if self.buffer.len() >= BYTES_PORT {
                        self.port = self.buffer.get_u16();
                        self.buffer.clear();
                        self.state = State::PayloadSize;
                    }
                }
                State::PayloadSize => {
                    pos = self.read_field(pos, BYTES_PAYLOAD_SIZE, buf);

                    if self.buffer.len() >= BYTES_PAYLOAD_SIZE {
                        self.payload_size = self.buffer.get_u16() as usize;
                        self.buffer.clear();
                        self.state = State::CRLF;
                    }
                }
                State::CRLF => {
                    pos = self.read_field(pos, BYTES_CRLF, buf);

                    if self.buffer.len() >= BYTES_CRLF {
                        self.buffer.clear();
                        self.state = State::Payload;
                    }
                }
                State::Payload => {
                    let size = self.payload_size;
                    pos = self.read_field(pos, size, buf);

                    // When we already have all the bytes
                    if self.buffer.len() >= size {
                        let destination =
                            match format!("{}:{}", self.addr.to_string(), self.port).parse() {
                                Ok(dest) => dest,
                                Err(e) => {
                                    return Poll::Ready(Err(Error::new(ErrorKind::InvalidData, e)))
                                }
                            };

                        match self
                            .udp_socket
                            .poll_send_to(cx, self.buffer.borrow(), destination)
                        {
                            Poll::Ready(res) => match res {
                                Ok(_) => (),
                                Err(e) => {
                                    warn!("Failed to write to remote: {}", e);
                                    return Poll::Ready(Err(e));
                                }
                            },
                            Poll::Pending => return Poll::Pending,
                        }
                        self.buffer.clear();
                        self.state = State::Atype;
                    }
                }
            }
        }

        return Poll::Ready(Ok(buf.len()));
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<()>> {
        return Poll::Ready(Ok(()));
    }

    fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<()>> {
        return Poll::Ready(Ok(()));
    }
}

impl PacketTrojanOutboundStream {
    pub async fn new() -> Result<Box<dyn OutboundStream>> {
        let stream = PacketTrojanOutboundStream {
            udp_socket: UdpSocket::bind("0.0.0.0:0").await.unwrap(),

            buffer: BytesMut::with_capacity(1024),
            state: State::Atype,

            atype: Atype::IPv4,
            addr: IpAddress::from_u32(0),
            addr_size: 0,
            port: 0,
            payload_size: 0,
        };
        Ok(Box::new(stream))
    }

    fn read_field(&mut self, pos: usize, size: usize, buf: &[u8]) -> usize {
        let cap = min(size - self.buffer.len(), buf.len() - pos);
        self.buffer.put_slice(&buf[pos..pos + cap]);
        pos + cap
    }
}
