use crate::protocol::common::addr::{IpAddress, IPV4_SIZE, IPV6_SIZE, IpAddrPort};
use crate::protocol::common::atype::Atype;
use crate::protocol::common::command::Command;
use crate::protocol::trojan::base::{Request, HEX_SIZE};
use crate::protocol::trojan::packet::TrojanUdpPacketHeader;

use bytes::Bytes;
use std::io::Result;
use tokio::io::{AsyncRead, AsyncReadExt};

pub async fn parse<T: AsyncRead + Unpin>(stream: &mut T) -> Result<Request> {
    // Read hex value for authentication
    let mut hex = vec![0u8; HEX_SIZE];
    stream.read_exact(&mut hex).await?;

    // Read CLRF
    stream.read_u16().await?;

    // Extract command
    let command = match Command::from(stream.read_u8().await?) {
        Ok(command) => command,
        Err(e) => return Err(e),
    };

    // Read address type
    let atype = match Atype::from(stream.read_u8().await?) {
        Ok(atype) => atype,
        Err(e) => return Err(e),
    };

    // Get address size and address object
    let (_, addr) = match atype {
        Atype::IPv4 => (IPV4_SIZE, IpAddress::from_u32(stream.read_u32().await?)),
        Atype::IPv6 => (IPV6_SIZE, IpAddress::from_u128(stream.read_u128().await?)),
        Atype::DomainName => {
            // Read domain name size
            let size = stream.read_u8().await? as usize;

            // Read domain name context
            let mut buf = vec![0u8; size];
            stream.read_exact(&mut buf).await?;
            (size, IpAddress::from_bytes(Bytes::from(buf)))
        }
    };

    // Read port number
    let port = stream.read_u16().await?;

    // Read CLRF
    stream.read_u16().await?;

    Ok(Request::new(
        hex,
        command,
        atype,
        addr,
        port,
        crate::proxy::base::SupportedProtocols::TROJAN,
    ))
}

pub async fn parse_udp<T: AsyncRead + Unpin>(reader: &mut T) -> Result<TrojanUdpPacketHeader> {
    // Read address type
    let atype = Atype::from(reader.read_u8().await?)?;

    // Read the address type
    let addr = match atype {
        Atype::IPv4 => IpAddress::from_u32(reader.read_u32().await?),
        Atype::IPv6 => IpAddress::from_u128(reader.read_u128().await?),
        Atype::DomainName => {
            // Get payload size
            let size = reader.read_u8().await? as usize;
            let mut buf = vec![0u8; size];

            // Read data into buffer
            reader.read_exact(&mut buf).await?;
            IpAddress::from_bytes(Bytes::from(buf))
        }
    };

    // Read port, payload length and CRLF
    let port = reader.read_u16().await?;
    let length = reader.read_u16().await?;
    reader.read_u16().await?;

    Ok(TrojanUdpPacketHeader {
        atype,
        dest: IpAddrPort::new(addr, port).into(),
        payload_size: length as usize
    })
}
