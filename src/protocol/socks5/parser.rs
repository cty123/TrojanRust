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
        _ => return Err(Error::new(ErrorKind::InvalidInput, "Unsupported version number")),
    };

    // Read command byte
    let command = match stream.read_u8().await? {
        CONNECT => CONNECT,
        BIND => BIND,
        UDP => UDP,
        _ => return Err(Error::new(ErrorKind::InvalidInput, "Unsupported command"))
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

    // Read the actual address, don't support domain name for now
    let addr = match atype {
        ATYPE_IPV4 => {
            let mut buf = [0u8; IPV4_SIZE];
            stream.read_exact(&mut buf).await?;
            IpAddress::IpAddr(IpAddr::V4(Ipv4Addr::new(buf[0], buf[1], buf[2], buf[3])))
        },
        ATYPE_IPV6 => {
            let mut buf = [0u8; IPV6_SIZE];
            stream.read_exact(&mut buf).await?;
            IpAddress::IpAddr(IpAddr::V6(Ipv6Addr::new(
                u16::from_be_bytes([buf[0], buf[1]]), 
                u16::from_be_bytes([buf[2], buf[3]]), 
                u16::from_be_bytes([buf[4], buf[5]]), 
                u16::from_be_bytes([buf[6], buf[7]]), 
                u16::from_be_bytes([buf[8], buf[9]]), 
                u16::from_be_bytes([buf[10], buf[11]]), 
                u16::from_be_bytes([buf[12], buf[13]]), 
                u16::from_be_bytes([buf[14], buf[15]])
            )))
        }
        // Temporarily not supporting domain name
        _ => return Err(Error::new(ErrorKind::InvalidInput, "Unsupported address type"))
    };

    // Read port number
    let port = stream.read_u16().await?;

    let request = Request::new(version, command, rsv, atype, port, addr);

    return Ok(request);
}
