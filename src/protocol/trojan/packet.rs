use log::{info, debug, warn};

use std::io::{Result, Error, ErrorKind};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::net::SocketAddr;
use std::cmp::min;
use std::convert::TryInto;
use std::borrow::{Borrow, BorrowMut};

use bytes::{BytesMut, BufMut, Buf};
use tokio::net::UdpSocket;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

use crate::protocol::common::addr::{ATYPE_IPV4, ATYPE_IPV6, ATYPE_DOMAIN_NAME, ipv4_to_string, ipv6_to_string, IPV4_SIZE, IPV6_SIZE};

const STATE_ATYPE: u8 = 0;
const STATE_ADDR_SIZE: u8 = 1;
const STATE_ADDR: u8 = 2;
const STATE_PORT: u8 = 3;
const STATE_PAYLOAD_SIZE: u8 = 4;
const STATE_CRLF: u8 = 5;
const STATE_PAYLOAD: u8 = 6;

const BYTES_ATYPE: usize = 1;
const BYTES_ADDR_SIZE: usize = 1;
const BYTES_PORT: usize = 2;
const BYTES_PAYLOAD_SIZE: usize = 2;
const BYTES_CRLF: usize = 2;


pub struct PacketTrojanOutboundStream {
    udp_socket: UdpSocket,

    // Internal states
    buffer: BytesMut,
    state: u8,

    // UDP request packet info
    atype: u8,
    addr: [u8; 256],
    addr_size: usize,
    port: u16,
    payload_size: usize,

    // Denotes the index of the payload that has been written
    payload_index: usize,
}

impl AsyncRead for PacketTrojanOutboundStream {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<Result<()>> {
        let mut header = BytesMut::with_capacity(512);
        let len = self.addr_size;
        header.put_u8(self.atype);
        header.put_slice(&self.addr[0..len]);
        header.put_u16(self.port);

        let mut b = [0; 4096];
        let mut readbuf = ReadBuf::new(&mut b);

        return match self.udp_socket.poll_recv_from(cx, &mut readbuf) {
            Poll::Ready(res) => {
                match res {
                    Ok(_) => {
                        info!("Read bytes {:?}", readbuf.filled());
                        header.put_u16(readbuf.filled().len() as u16);
                        header.put_u8(0x0d);
                        header.put_u8(0x0a);
                        buf.put_slice(&header.to_vec());
                        buf.put_slice(readbuf.filled());
                        Poll::Ready(Ok(()))
                    }
                    Err(e) => Poll::Ready(Err(e))
                }
            }
            Poll::Pending => Poll::Pending
        };
    }
}

impl AsyncWrite for PacketTrojanOutboundStream {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        // Sanity check
        if buf.is_empty() {
            return Poll::Ready(Ok(0));
        }

        let mut ctr: usize = 0;

        while ctr < buf.len() {
            match self.state {
                STATE_ATYPE => {
                    self.atype = buf[ctr];
                    self.state = STATE_ADDR_SIZE;
                    ctr += 1;
                }
                STATE_ADDR_SIZE => {
                    match self.atype {
                        ATYPE_IPV4 => {
                            self.addr_size = IPV4_SIZE;
                            self.state = STATE_ADDR;
                            continue;
                        }
                        ATYPE_IPV6 => {
                            self.addr_size = IPV6_SIZE;
                            self.state = STATE_ADDR;
                            continue;
                        }
                        ATYPE_DOMAIN_NAME => {
                            self.addr_size = usize::from(buf[ctr]);
                            self.state = STATE_ADDR;
                            ctr += 1;
                        }
                        _ => return Poll::Ready(Err(Error::new(ErrorKind::InvalidInput, "Unsupported address type")))
                    }
                }
                STATE_ADDR => {
                    let march_size = min(self.addr_size - self.buffer.len(), buf.len() - ctr);
                    self.buffer.put_slice(&buf[ctr..ctr + march_size]);

                    // If need to wait for more bytes to come
                    if self.buffer.len() + march_size < self.addr_size {
                        ctr += march_size;
                        continue;
                    }

                    // When we already have all the bytes
                    let size = self.addr_size;
                    let addr_vec = self.buffer.to_vec();
                    self.addr[0..size].copy_from_slice(&addr_vec);
                    self.buffer.clear();
                    self.state = STATE_PORT;
                    ctr += march_size;
                }
                STATE_PORT => {
                    let march_size = min(BYTES_PORT - self.buffer.len(), buf.len() - ctr);
                    self.buffer.put_slice(&buf[ctr..ctr + march_size]);

                    // If need to wait for more bytes to come
                    if self.buffer.len() + march_size < BYTES_PORT {
                        ctr += march_size;
                        continue;
                    }

                    // When we already have all the bytes
                    self.port = self.buffer.get_u16();
                    self.buffer.clear();
                    self.state = STATE_PAYLOAD_SIZE;
                    ctr += march_size;
                }
                STATE_PAYLOAD_SIZE => {
                    let march_size = min(BYTES_PAYLOAD_SIZE - self.buffer.len(), buf.len() - ctr);
                    self.buffer.put_slice(&buf[ctr..ctr + march_size]);

                    // If need to wait for more bytes to come
                    if self.buffer.len() + march_size < BYTES_PORT {
                        ctr += march_size;
                        continue;
                    }

                    // When we already have all the bytes
                    self.payload_size = self.buffer.get_u16() as usize;
                    self.buffer.clear();
                    self.state = STATE_CRLF;
                    ctr += march_size;
                }
                STATE_CRLF => {
                    let march_size = min(BYTES_CRLF - self.buffer.len(), buf.len() - ctr);
                    self.buffer.put_slice(&buf[ctr..ctr + march_size]);

                    // If need to wait for more bytes to come
                    if self.buffer.len() + march_size < BYTES_PORT {
                        ctr += march_size;
                        continue;
                    }

                    // When we already have all the bytes
                    self.buffer.clear();
                    self.state = STATE_PAYLOAD;
                    ctr += march_size;

                    info!("Read trojan udp request header [Len: {}:{:?}:{}]", self.payload_size, &self.addr[0..4], self.port);
                }
                STATE_PAYLOAD => {
                    let march_size = min(self.payload_size - self.buffer.len(), buf.len() - ctr);
                    self.buffer.put_slice(&buf[ctr..ctr + march_size]);

                    // If need to wait for more bytes to come
                    if self.buffer.len() + march_size < BYTES_PORT {
                        ctr += march_size;
                        continue;
                    }

                    let addr = match self.atype {
                        ATYPE_IPV4 => ipv4_to_string(self.addr[0..IPV4_SIZE].try_into().unwrap()),
                        ATYPE_IPV6 => ipv6_to_string(self.addr[0..IPV6_SIZE].try_into().unwrap()),
                        _ => return Poll::Ready(Err(Error::new(ErrorKind::InvalidInput, "Unsupported address type")))
                    };

                    // When we already have all the bytes
                    let destination = format!("{}:{}", addr, self.port).parse().unwrap();
                    match self.udp_socket.poll_send_to(cx, self.buffer.borrow(), destination) {
                        Poll::Ready(res) => match res {
                            Ok(n) => info!("Forwarded bytes {:?}", self.buffer.to_vec()),
                            Err(e) => {
                                warn!("Failed to write: {}", e);
                                return Poll::Ready(Err(e));
                            }
                        }
                        Poll::Pending => return Poll::Pending
                    }
                    self.buffer.clear();
                    self.state = STATE_ATYPE;
                    ctr += march_size;
                }
                _ => return Poll::Ready(Err(Error::new(ErrorKind::InvalidData, "Incorrect connection state")))
            }
        }

        return Poll::Ready(Ok(buf.len()));
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        return Poll::Ready(Ok(()));
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        return Poll::Ready(Ok(()));
    }
}

impl PacketTrojanOutboundStream {
    pub async fn new() -> Result<PacketTrojanOutboundStream> {
        let stream = PacketTrojanOutboundStream {
            udp_socket: UdpSocket::bind("0.0.0.0:0").await.unwrap(),

            buffer: BytesMut::with_capacity(1024),
            state: STATE_ATYPE,

            atype: ATYPE_IPV4,
            addr: [0; 256],
            addr_size: 0,
            port: 0,
            payload_size: 0,

            payload_index: 0,
        };
        Ok(stream)
    }
}

// pub struct PacketTrojanInboundStream<IO> {
//     stream: IO,
// }
//
// impl<IO> PacketTrojanInboundStream<IO>
//     where
//         IO: AsyncRead + AsyncWrite + Unpin
// {
//     pub fn new(stream: IO) -> PacketTrojanInboundStream<IO> {
//         return PacketTrojanInboundStream {
//             stream
//         }
//     }
// }
