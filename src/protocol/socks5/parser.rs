use std::io::{Error, ErrorKind};
use std::io::Result;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt};

use crate::protocol::common::addr::IpAddress;
use crate::protocol::socks5::base::Request;
use crate::protocol::common::command::{CONNECT, BIND, UDP};
use crate::protocol::common::addr::{ATYPE_IPV4, ATYPE_DOMAIN_NAME, ATYPE_IPV6, IPV4_SIZE, IPV6_SIZE};

const VERSION: u8 = 5;

pub async fn parse<IO>(mut stream: IO) -> Result<Request>
    where
        IO: AsyncRead + AsyncWrite + Unpin
{
    // Read version number
    let version = match stream.read_u8().await? {
        VERSION => VERSION,
        _ => return Err(Error::new(ErrorKind::Unsupported, "Unsupported version number")),
    };

    // Read command byte
    let command = match stream.read_u8().await? {
        CONNECT => CONNECT,
        BIND => BIND,
        UDP => UDP,
        _ => return Err(Error::new(ErrorKind::Unsupported, "Unsupported command"))
    };

    // Don't do anything about rsv
    let rsv = stream.read_u8().await?;

    // Read address type
    let atype = match stream.read_u8().await? {
        ATYPE_IPV4 => ATYPE_IPV4,
        ATYPE_DOMAIN_NAME => ATYPE_DOMAIN_NAME,
        ATYPE_IPV6 => ATYPE_IPV6,
        _ => return Err(Error::new(ErrorKind::InvalidInput, "Unsupported address type"))
    };    
    
    // Get address size
    let addr_size = match atype {
        ATYPE_IPV4 => IPV4_SIZE,
        ATYPE_IPV6 => IPV6_SIZE,
        ATYPE_DOMAIN_NAME => usize::from(stream.read_u8().await?),
        _ => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Unsupported address type",
            ))
        }
    };

    // Read the actual address, don't support domain name for now
    let addr = match atype {
        ATYPE_IPV4 => IpAddress::from_u32(stream.read_u32().await?),
        ATYPE_IPV6 => IpAddress::from_u128(stream.read_u128().await?),
        ATYPE_DOMAIN_NAME => {
            let mut buf = [0u8; 256];
            stream.read_exact(&mut buf[..addr_size]).await?;
            IpAddress::from_vec(buf[..addr_size].to_vec())
        }
        // Temporarily not supporting domain name
        _ => return Err(Error::new(ErrorKind::InvalidInput, "Unsupported address type"))
    };

    // Read port number
    let port = stream.read_u16().await?;

    let request = Request::new(version, command, rsv, atype, port, addr);

    return Ok(request);
}
