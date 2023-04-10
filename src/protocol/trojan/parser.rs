use crate::protocol::common::addr::{IpAddrPort, IpAddress, IPV4_SIZE, IPV6_SIZE};
use crate::protocol::common::atype::Atype;
use crate::protocol::common::command::Command;
use crate::protocol::trojan::base::{Request, HEX_SIZE};
use crate::protocol::trojan::packet::TrojanUdpPacketHeader;

use bytes::{Bytes, BytesMut};
use constant_time_eq::constant_time_eq;
use std::io::{Error, ErrorKind, Result};
use tokio::io::{AsyncRead, AsyncReadExt};

pub async fn parse_and_authenticate<T: AsyncRead + Unpin>(
    stream: &mut T,
    hex_key: &[u8],
) -> Result<Request> {
    // Read hex value for authentication.
    let mut hex = BytesMut::with_capacity(HEX_SIZE);
    let n = stream.read_buf(&mut hex).await?;
    if n != HEX_SIZE {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Malformed Trojan header",
        ));
    }

    // Authentication needs to happen here, otherwise the following read operations are not safe.
    if !constant_time_eq(hex_key, &hex) {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Trojan password mismatch",
        ));
    }

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

    // Resolve DNS name if the requested address is DNS name.
    let dest = match IpAddrPort::new(addr, port).into() {
        Ok(d) => d,
        Err(e) => return Err(e),
    };

    Ok(TrojanUdpPacketHeader {
        atype,
        dest,
        payload_size: length as usize,
    })
}
