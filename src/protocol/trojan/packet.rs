use crate::protocol::common::addr::IpAddress;
use crate::protocol::common::atype::Atype;
use crate::protocol::common::request::InboundRequest;
use crate::protocol::trojan::base::CRLF;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use futures::{Sink, SinkExt, Stream};
use log::warn;
use std::io::{Error, ErrorKind};
use std::net::IpAddr;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::UdpSocket;
use tokio_stream::StreamExt;
use tokio_util::codec::{Decoder, Encoder};

/// Trojan UDP packet header size may vary due to the type of IPAddress to be proxied:
/// Type    | Atype  | Address        | Port   | Payload Size | CRLF   |
/// --------|--------|----------------|--------|--------------|--------|
/// IPv4:   | 1 byte | 4 byte         | 2 byte | 2 byte       | 2 byte |
/// IPv4:   | 1 byte | 16 byte        | 2 byte | 2 byte       | 2 byte |
/// Domain: | 1 byte | 1 - 255 byte   | 2 byte | 2 byte       | 2 byte |
const IPV4_HEADER_SIZE: usize = 11;
const IPV6_HEADER_SIZE: usize = 23;

/// Define the size of the buffer used to transport the data back and forth
const BUF_SIZE: usize = 4096;

/// According the official documentation for Trojan protocol, the UDP data will be segmented into Trojan UDP packets,
/// which allows the outbound handler to also forward them as real UDP packets to the desired destinations.
/// Link: https://trojan-gfw.github.io/trojan/protocol.html
pub struct TrojanUdpPacket {
    pub atype: Atype,
    pub dest: IpAddress,
    pub port: u16,
    pub payload: Bytes,
}

/// TrojanUdpPacketCodec used to encode and decode between Trojan UDP packet and raw bytes data.
pub struct TrojanUdpPacketCodec {}

impl TrojanUdpPacketCodec {
    #[inline]
    pub fn new() -> Self {
        return TrojanUdpPacketCodec {};
    }
}

impl Encoder<TrojanUdpPacket> for TrojanUdpPacketCodec {
    type Error = std::io::Error;

    fn encode(
        &mut self,
        item: TrojanUdpPacket,
        dst: &mut bytes::BytesMut,
    ) -> Result<(), Self::Error> {
        // Write address type to output buffer
        dst.put_u8(item.atype as u8);

        // Write entire IP address octets to buffer
        match item.dest {
            IpAddress::IpAddr(IpAddr::V4(addr)) => {
                dst.put_slice(&addr.octets());
            }
            IpAddress::IpAddr(IpAddr::V6(addr)) => {
                dst.put_slice(&addr.octets());
            }
            IpAddress::Domain(ref domain) => {
                dst.put_u8(domain.as_bytes().len() as u8);
                dst.put_slice(domain.as_bytes());
            }
        };

        // Write port, payload length, CRLF and payload body
        dst.put_u16(item.port);
        dst.put_u16(item.payload.len() as u16);
        dst.put_u16(CRLF);
        dst.put_slice(&item.payload);

        return Ok(());
    }
}

impl Decoder for TrojanUdpPacketCodec {
    type Item = TrojanUdpPacket;

    type Error = std::io::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // A valid UDP packet needs to have at least:
        // Atype(1 byte) + Address(4 byte at least) + Port(2 byte) + Length(2 byte) + CRLF(2 byte) = 11 bytes
        if src.len() < IPV4_HEADER_SIZE {
            src.reserve(IPV4_HEADER_SIZE);
            return Ok(None);
        }

        // Need to get address type to be able to determine the size of the packet
        let atype = match Atype::from(src[0]) {
            Ok(t) => t,
            Err(_) => {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidData,
                    "Invalid address type",
                ))
            }
        };

        // Get expected header size
        let header_size = match atype {
            // Expect header to be 11 bytes
            Atype::IPv4 => IPV4_HEADER_SIZE,
            // Expect header to be 23 bytes
            Atype::IPv6 => IPV6_HEADER_SIZE,
            // Need to read extra byte to check the header size
            Atype::DomainName => {
                let addr_size = src[1] as usize;
                1 + 1 + addr_size + 2 + 2 + 2
            }
        };

        // Need more data in order to decode
        if src.len() < header_size {
            src.reserve(header_size);
            return Ok(None);
        }

        // Compute the packet size
        let payload_size =
            u16::from_be_bytes([src[header_size - 4], src[header_size - 3]]) as usize;

        // Need to wait until the entire packet to be present
        if src.len() < header_size + payload_size {
            src.reserve(payload_size);
            return Ok(None);
        }

        // Start the actual parsing
        src.advance(1);

        // Read address from buffer
        let address = match atype {
            Atype::IPv4 => IpAddress::from_u32(src.get_u32()),
            Atype::IPv6 => IpAddress::from_u128(src.get_u128()),
            Atype::DomainName => {
                let len = src.get_u8();
                let buf = src.copy_to_bytes(len as usize);
                IpAddress::from_bytes(buf)
            }
        };

        // Read port and packet length
        let (port, _length) = (src.get_u16(), src.get_u16());

        // Read CRLF
        src.advance(2);

        // Read payload from source buffer
        let payload = src.copy_to_bytes(payload_size);

        // Reserve header size for the next packet
        src.reserve(IPV4_HEADER_SIZE);

        return Ok(Some(TrojanUdpPacket {
            atype,
            dest: address,
            port,
            payload,
        }));
    }
}

/// Helper function to transport data from UDP packet stream to tokio UDP socket
pub async fn packet_stream_client_udp<R: Stream<Item = Result<TrojanUdpPacket, Error>> + Unpin>(
    mut packet_reader: R,
    socket: &UdpSocket,
) {
    loop {
        // Read next packet off the client trojan stream
        let packet = match packet_reader.next().await {
            None => continue,
            Some(res) => {
                match res {
                    Ok(p) => p,
                    Err(e) => {
                        warn!("Encountered error while reading the trojan UDP packets from client: {}", e);
                        return;
                    }
                }
            }
        };

        // Forward the received packet to remote
        match socket
            .send_to(&packet.payload, (packet.dest.to_string(), packet.port))
            .await
        {
            Ok(_) => (),
            Err(e) => {
                warn!(
                    "Encountered error while sending the UDP packets to remote server: {}",
                    e
                );
                return;
            }
        }
    }
}

/// Helper function to transport data from tokio UDP socket to Trojan UDP packet sink.
pub async fn packet_stream_server_udp<W: Sink<TrojanUdpPacket> + Unpin>(
    socket: &UdpSocket,
    mut packet_writer: W,
) where
    <W as futures::Sink<TrojanUdpPacket>>::Error: std::fmt::Display,
{
    loop {
        let mut buf = vec![0u8; BUF_SIZE];

        let (size, dest) = match socket.recv_from(&mut buf).await {
            Ok((s, d)) => (s, d),
            Err(e) => {
                warn!(
                    "Encountered error while reading the UDP packets from remote server: {}",
                    e
                );
                return;
            }
        };

        buf.truncate(size);

        match packet_writer
            .send(TrojanUdpPacket {
                atype: Atype::IPv4,
                dest: IpAddress::IpAddr(dest.ip()),
                port: dest.port(),
                payload: Bytes::from(buf),
            })
            .await
        {
            Ok(_) => (),
            Err(e) => {
                warn!(
                    "Encountered error while sending the trojan UDP packets to client: {}",
                    e
                );
                return;
            }
        }
    }
}

/// Helper function to transport Trojan UDP packet stream to the destination AsyncWrite stream.
pub async fn packet_stream_server_tcp<
    R: Stream<Item = Result<TrojanUdpPacket, Error>> + Unpin,
    W: AsyncWrite + Unpin,
>(
    mut packet_reader: R,
    mut server_writer: W,
) {
    loop {
        // Read next packet off the client trojan stream
        let packet = match packet_reader.next().await {
            None => continue,
            Some(res) => match res {
                Ok(p) => p,
                Err(e) => {
                    warn!(
                        "Encountered error while reading the trojan UDP packet from client: {}",
                        e
                    );
                    return;
                }
            },
        };

        // Forward the received packet to remote
        match server_writer.write_all(&packet.payload).await {
            Ok(_) => (),
            Err(e) => {
                warn!(
                    "Encountered error while sending the trojan UDP packet payload to remote server: {}",
                    e
                );
                return;
            }
        }
    }
}

/// Helper function to transport proxy response data from an AsyncRead stream to the client TrojanUdpPacket sink.
pub async fn packet_stream_client_tcp<R: AsyncRead + Unpin, W: Sink<TrojanUdpPacket> + Unpin>(
    mut server_reader: R,
    mut packet_writer: W,
    request: InboundRequest,
) where
    <W as futures::Sink<TrojanUdpPacket>>::Error: std::fmt::Display,
{
    loop {
        let mut buf = BytesMut::with_capacity(BUF_SIZE);

        let _size = match server_reader.read_buf(&mut buf).await {
            Ok(s) => s,
            Err(e) => {
                warn!(
                    "Encountered error while reading the UDP packets from remote server: {}",
                    e
                );
                return;
            }
        };

        warn!("buf length: {}", buf.len());

        match packet_writer
            .send(TrojanUdpPacket {
                atype: Atype::IPv4,
                dest: request.addr_port.ip.clone(),
                port: request.addr_port.port,
                payload: buf.freeze(),
            })
            .await
        {
            Ok(_) => (),
            Err(e) => {
                warn!(
                    "Encountered error while sending the trojan UDP packets to client: {}",
                    e
                );
                return;
            }
        }
    }
}
