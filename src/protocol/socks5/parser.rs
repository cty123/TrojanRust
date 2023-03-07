use crate::protocol::common::addr::IpAddress;
use crate::protocol::common::atype::Atype;
use crate::protocol::common::command::Command;
use crate::protocol::socks5::base::{Request, VERSION};

use bytes::Bytes;
use std::io::Result;
use std::io::{Error, ErrorKind};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};

pub async fn parse<T: AsyncRead + AsyncWrite + Unpin>(mut stream: T) -> Result<Request> {
    // Read version number
    let version = match stream.read_u8().await? {
        VERSION => VERSION,
        _ => {
            return Err(Error::new(
                ErrorKind::Unsupported,
                "Unsupported version number",
            ))
        }
    };

    // Read command byte
    let command = match Command::from(stream.read_u8().await?) {
        Ok(command) => command,
        Err(e) => return Err(e),
    };

    // Don't do anything about rsv
    let rsv = stream.read_u8().await?;

    // Read address type
    let atype = match Atype::from(stream.read_u8().await?) {
        Ok(atype) => atype,
        Err(e) => return Err(e),
    };

    // Get address size and address object
    let addr = match atype {
        Atype::IPv4 => IpAddress::from_u32(stream.read_u32().await?),
        Atype::IPv6 => IpAddress::from_u128(stream.read_u128().await?),
        Atype::DomainName => {
            // Read address size
            let size = stream.read_u8().await? as usize;
            let mut buf = vec![0u8; size];

            // Read address data
            stream.read_exact(&mut buf).await?;
            IpAddress::from_bytes(Bytes::from(buf))
        }
    };

    // Read port number
    let port = stream.read_u16().await?;

    Ok(Request::new(version, command, rsv, atype, port, addr))
}
