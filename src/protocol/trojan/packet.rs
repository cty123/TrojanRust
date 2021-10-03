use std::cmp::min;
use std::io::{Error, ErrorKind, Result};
use std::net::IpAddr;
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
    payload_ctr: usize,

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
        match &self.addr {
            IpAddress::IpAddr(IpAddr::V4(ipv4)) => {
                buf.put_slice(&ipv4.octets());
            }
            IpAddress::IpAddr(IpAddr::V6(ipv6)) => {
                buf.put_slice(&ipv6.octets());
            }
            IpAddress::Domain(domain) => {
                buf.put_slice(&domain.to_bytes());
            }
        }
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
                    pos = self.consume_payload(pos, BYTES_ATYPE, buf);
                    if self.buffer.len() >= BYTES_ATYPE {
                        self.atype = match Atype::from(self.buffer.get_u8()) {
                            Ok(atype) => atype,
                            Err(e) => return Poll::Ready(Err(e)),
                        };
                        self.buffer.clear();
                        self.state = State::AddrSize;
                    }
                }
                State::AddrSize => {
                    self.addr_size = match self.atype {
                        Atype::IPv4 => IPV4_SIZE,
                        Atype::IPv6 => IPV6_SIZE,
                        Atype::DomainName => {
                            let size = usize::from(buf[pos]);
                            pos += 1;
                            size
                        }
                    };
                    self.state = State::Addr;
                }
                State::Addr => {
                    let addr_size = self.addr_size;
                    pos = self.consume_payload(pos, addr_size, buf);
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
                    pos = self.consume_payload(pos, BYTES_PORT, buf);
                    if self.buffer.len() >= BYTES_PORT {
                        self.port = self.buffer.get_u16();
                        self.buffer.clear();
                        self.state = State::PayloadSize;
                    }
                }
                State::PayloadSize => {
                    pos = self.consume_payload(pos, BYTES_PAYLOAD_SIZE, buf);
                    if self.buffer.len() >= BYTES_PAYLOAD_SIZE {
                        self.payload_size = self.buffer.get_u16() as usize;
                        self.buffer.clear();
                        self.state = State::CRLF;
                    }
                }
                State::CRLF => {
                    pos = self.consume_payload(pos, BYTES_CRLF, buf);
                    if self.buffer.len() >= BYTES_CRLF {
                        self.buffer.clear();
                        self.state = State::Payload;
                    }
                }
                State::Payload => {
                    // If all payloads have been written we can reset the state machine to handle the next packet
                    if self.payload_ctr >= self.payload_size {
                        self.reset();
                        continue;
                    }

                    // Otherwise, if we still have payloads to write we need to get the destination first
                    let destination = match format!("{}:{}", self.addr.to_string(), self.port)
                        .parse()
                    {
                        Ok(dest) => dest,
                        Err(e) => return Poll::Ready(Err(Error::new(ErrorKind::InvalidData, e))),
                    };

                    // Actually write bytes through UDP, and increment counters
                    // TODO: Use poll_send here so that we don't need to parse destination repeatedly
                    match self.udp_socket.poll_send_to(cx, &buf[pos..], destination) {
                        Poll::Ready(res) => match res {
                            Ok(n) => {
                                pos += n;
                                self.payload_ctr += n;
                            }
                            Err(e) => {
                                warn!("Failed to write to remote: {}", e);
                                return Poll::Ready(Err(e));
                            }
                        },
                        Poll::Pending => return Poll::Pending,
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
            udp_socket: UdpSocket::bind("0.0.0.0:0").await?,

            buffer: BytesMut::with_capacity(1024),
            state: State::Atype,
            payload_ctr: 0,

            atype: Atype::IPv4,
            addr: IpAddress::from_u32(0),
            addr_size: 0,
            port: 0,
            payload_size: 0,
        };
        Ok(Box::new(stream))
    }

    /// Helper function to consume the payload passed by poll_write function call. The function takes
    /// the currently consuming position of the payload, total amount of bytes the buffer should store
    /// before returning and the payload.
    #[inline]
    fn consume_payload(&mut self, pos: usize, size: usize, payload: &[u8]) -> usize {
        // Read at most cap bytes, because in each state the state machine needs to decide
        // how many more bytes it needs to read next based on what is in the buffer. So here
        // we read at most cap bytes and give it back to the caller to decide how many more
        // to read.
        let cap = min(size - self.buffer.len(), payload.len() - pos);

        // Copy the payload to buffer for the caller. This is not the most efficient way of writing
        // but generally speaking the Trojan header size are small compared to the payload, and we
        // only copy for headers.
        self.buffer.put_slice(&payload[pos..pos + cap]);

        // March pos to the next position of the payload. The caller is supposed to update its own
        // counter of the current position in the payload based on the return value here.
        pos + cap
    }

    /// Helper function to reset the internal state of the stream to be able to accept the next packet.
    fn reset(&mut self) {
        self.payload_ctr = 0;
        self.buffer.clear();
        self.state = State::Atype;
    }
}
