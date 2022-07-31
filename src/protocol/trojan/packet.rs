use crate::protocol::common::addr::{IpAddrPort, IpAddress};
use crate::protocol::common::atype::Atype;
use crate::protocol::common::request::InboundRequest;
use crate::protocol::trojan::base::CRLF;
use crate::protocol::trojan::parser::parse_udp;

use log::debug;
use std::io;
use std::net::{IpAddr, SocketAddr};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::UdpSocket;

/// Define the size of the buffer used to transport the data back and forth
const BUF_SIZE: usize = 4096;

/// According the official documentation for Trojan protocol, the UDP data will be segmented into Trojan UDP packets,
/// which allows the outbound handler to also forward them as real UDP packets to the desired destinations.
/// Link: https://trojan-gfw.github.io/trojan/protocol.html
pub struct TrojanUdpPacketHeader {
    pub atype: Atype,
    pub dest: SocketAddr,
    pub payload_size: usize,
}

pub async fn copy_client_reader_to_udp_socket<R: AsyncRead + Unpin>(
    client_reader: R,
    server_writer: &UdpSocket,
) -> io::Result<()> {
    let mut read_buf = vec![0u8; BUF_SIZE];
    let mut client_reader = BufReader::new(client_reader);

    loop {
        let header = parse_udp(&mut client_reader).await?;

        debug!(
            "Forwarding {} bytes to {}",
            header.payload_size, header.dest
        );

        assert!(
            header.payload_size <= BUF_SIZE,
            "Payload size exceeds read buffer size"
        );

        let size = client_reader
            .read_exact(&mut read_buf[..header.payload_size])
            .await?;

        assert!(
            size == header.payload_size,
            "Failed to read the entire trojan udp packet, expect: {} bytes, read: {} bytes",
            header.payload_size,
            size
        );

        server_writer
            .send_to(&read_buf[..header.payload_size], header.dest)
            .await?;
    }
}

pub async fn copy_udp_socket_to_client_writer<W: AsyncWrite + Unpin>(
    server_reader: &UdpSocket,
    client_writer: W,
    addr: IpAddrPort,
) -> io::Result<()> {
    let mut read_buf = vec![0u8; BUF_SIZE];
    let mut client_writer = BufWriter::new(client_writer);
    let (addr, port) = (addr.ip, addr.port);

    loop {
        let (size, _dest) = server_reader.recv_from(&mut read_buf).await?;

        match addr {
            IpAddress::IpAddr(IpAddr::V4(addr)) => {
                client_writer.write_u8(Atype::IPv4 as u8).await?;
                client_writer.write_all(&addr.octets()).await?;
            }
            IpAddress::IpAddr(IpAddr::V6(addr)) => {
                client_writer.write_u8(Atype::IPv6 as u8).await?;
                client_writer.write_all(&addr.octets()).await?;
            }
            IpAddress::Domain(ref domain) => {
                client_writer.write_u8(Atype::DomainName as u8).await?;
                client_writer
                    .write_u8(domain.as_bytes().len() as u8)
                    .await?;
                client_writer.write_all(domain.as_bytes()).await?;
            }
        }

        client_writer.write_u16(port).await?;
        client_writer.write_u16(size as u16).await?;
        client_writer.write_u16(CRLF).await?;
        client_writer.write_all(&read_buf[..size]).await?;
        client_writer.flush().await?;
    }
}

pub async fn copy_client_reader_to_udp_server_writer<
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
>(
    mut client_reader: R,
    server_writer: W,
    request: InboundRequest,
) -> io::Result<()> {
    let mut read_buf = vec![0u8; BUF_SIZE];
    let mut server_writer = BufWriter::new(server_writer);

    loop {
        let size = client_reader.read(&mut read_buf).await?;

        server_writer.write_u8(request.atype as u8).await?;

        match request.addr_port.ip {
            IpAddress::IpAddr(IpAddr::V4(addr)) => {
                server_writer.write_all(&addr.octets()).await?;
            }
            IpAddress::IpAddr(IpAddr::V6(addr)) => {
                server_writer.write_all(&addr.octets()).await?;
            }
            IpAddress::Domain(ref domain) => {
                server_writer
                    .write_u8(domain.as_bytes().len() as u8)
                    .await?;
                server_writer.write_all(domain.as_bytes()).await?;
            }
        }

        server_writer.write_u16(request.addr_port.port).await?;
        server_writer.write_u16(size as u16).await?;
        server_writer.write_u16(CRLF).await?;
        server_writer.write_all(&read_buf[..size]).await?;
        server_writer.flush().await?;
    }
}

pub async fn copy_udp_server_reader_to_client_writer<
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
>(
    server_reader: R,
    mut client_writer: W,
) -> io::Result<()> {
    let mut read_buf = vec![0u8; BUF_SIZE];
    let mut server_reader = BufReader::new(server_reader);

    loop {
        let header = parse_udp(&mut server_reader).await?;

        server_reader
            .read_exact(&mut read_buf[..header.payload_size])
            .await?;

        client_writer
            .write_all(&read_buf[..header.payload_size])
            .await?;
    }
}

// /// Helper function to transport data from UDP packet stream to tokio UDP socket
// pub async fn packet_stream_client_udp<R: Stream<Item = Result<TrojanUdpPacket, Error>> + Unpin>(
//     mut packet_reader: R,
//     socket: &UdpSocket,
// ) {
//     loop {
//         // Read next packet off the client trojan stream
//         let packet = match packet_reader.next().await {
//             None => continue,
//             Some(res) => {
//                 match res {
//                     Ok(p) => p,
//                     Err(e) => {
//                         warn!("Encountered error while reading the trojan UDP packets from client: {}", e);
//                         return;
//                     }
//                 }
//             }
//         };

//         // Forward the received packet to remote
//         match socket
//             .send_to(&packet.payload, (packet.dest.to_string(), packet.port))
//             .await
//         {
//             Ok(_) => (),
//             Err(e) => {
//                 warn!(
//                     "Encountered error while sending the UDP packets to remote server: {}",
//                     e
//                 );
//                 return;
//             }
//         }
//     }
// }

// /// Helper function to transport data from tokio UDP socket to Trojan UDP packet sink.
// pub async fn packet_stream_server_udp<W: Sink<TrojanUdpPacket> + Unpin>(
//     socket: &UdpSocket,
//     mut packet_writer: W,
// ) where
//     <W as futures::Sink<TrojanUdpPacket>>::Error: std::fmt::Display,
// {
//     loop {
//         let mut buf = vec![0u8; BUF_SIZE];

//         let (size, dest) = match socket.recv_from(&mut buf).await {
//             Ok((s, d)) => (s, d),
//             Err(e) => {
//                 warn!(
//                     "Encountered error while reading the UDP packets from remote server: {}",
//                     e
//                 );
//                 return;
//             }
//         };

//         buf.truncate(size);

//         match packet_writer
//             .send(TrojanUdpPacket {
//                 atype: Atype::IPv4,
//                 dest: IpAddress::IpAddr(dest.ip()),
//                 port: dest.port(),
//                 payload: Bytes::from(buf),
//             })
//             .await
//         {
//             Ok(_) => (),
//             Err(e) => {
//                 warn!(
//                     "Encountered error while sending the trojan UDP packets to client: {}",
//                     e
//                 );
//                 return;
//             }
//         }
//     }
// }

// /// Helper function to transport Trojan UDP packet stream to the destination AsyncWrite stream.
// pub async fn packet_stream_server_tcp<
//     R: Stream<Item = Result<TrojanUdpPacket, Error>> + Unpin,
//     W: AsyncWrite + Unpin,
// >(
//     mut packet_reader: R,
//     mut server_writer: W,
// ) -> io::Result<()> {
//     loop {
//         // Read next packet off the client trojan stream
//         let packet = match packet_reader.next().await {
//             None => continue,
//             Some(res) => match res {
//                 Ok(p) => p,
//                 Err(e) => {
//                     warn!(
//                         "Encountered error while reading the trojan UDP packet from client: {}",
//                         e
//                     );
//                     return Err(Error::new(
//                         ErrorKind::ConnectionReset,
//                         "Failed to read UDP packet from client",
//                     ));
//                 }
//             },
//         };

//         // Forward the received packet to remote
//         match server_writer.write_all(&packet.payload).await {
//             Ok(_) => (),
//             Err(e) => {
//                 warn!(
//                     "Encountered error while sending the trojan UDP packet payload to remote server: {}",
//                     e
//                 );
//                 return Err(Error::new(
//                     ErrorKind::ConnectionRefused,
//                     "Failed to send UDP packet to server",
//                 ));
//             }
//         }
//     }
// }

// /// Helper function to transport proxy response data from an AsyncRead stream to the client TrojanUdpPacket sink.
// pub async fn packet_stream_client_tcp<R: AsyncRead + Unpin, W: Sink<TrojanUdpPacket> + Unpin>(
//     mut server_reader: R,
//     mut packet_writer: W,
//     request: InboundRequest,
// ) -> io::Result<()>
// where
//     <W as futures::Sink<TrojanUdpPacket>>::Error: std::fmt::Display,
// {
//     loop {
//         let mut buf = vec![0u8; BUF_SIZE];

//         let size = match server_reader.read(&mut buf).await {
//             Ok(s) => s,
//             Err(e) => {
//                 warn!(
//                     "Encountered error while reading the UDP packets from remote server: {}",
//                     e
//                 );
//                 return Err(Error::new(
//                     ErrorKind::ConnectionReset,
//                     "Failed to read UDP packet from server",
//                 ));
//             }
//         };

//         buf.truncate(size);

//         match packet_writer
//             .send(TrojanUdpPacket {
//                 atype: Atype::IPv4,
//                 dest: request.addr_port.ip.clone(),
//                 port: request.addr_port.port,
//                 payload: Bytes::from(buf),
//             })
//             .await
//         {
//             Ok(_) => (),
//             Err(e) => {
//                 warn!(
//                     "Encountered error while sending the trojan UDP packets to client: {}",
//                     e
//                 );
//                 return Err(Error::new(
//                     ErrorKind::ConnectionRefused,
//                     "Failed to write trojan UDP packet to client",
//                 ));
//             }
//         }
//     }
// }
