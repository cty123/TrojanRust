use std::io::{Error, ErrorKind, Result};

use log::debug;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::protocol::common::addr::{
    IpAddress, ATYPE_DOMAIN_NAME, ATYPE_IPV4, ATYPE_IPV6, IPV4_SIZE, IPV6_SIZE,
};
use crate::protocol::common::command::{CONNECT, UDP};
use crate::protocol::trojan::base::Request;

pub async fn parse<IO>(mut stream: IO) -> Result<Request>
where
    IO: AsyncReadExt + AsyncWriteExt + Unpin,
{
    // Read hex value for authentication
    let mut hex = [0; 56];
    stream.read_exact(&mut hex).await?;

    // Read CLRF
    stream.read_u16().await?;

    // Extract command
    let command = match stream.read_u8().await? {
        CONNECT => CONNECT,
        UDP => UDP,
        _ => return Err(Error::new(ErrorKind::InvalidInput, "Unsupported command")),
    };

    // Extract address type
    let atype = match stream.read_u8().await? {
        ATYPE_IPV4 => ATYPE_IPV4,
        ATYPE_DOMAIN_NAME => ATYPE_DOMAIN_NAME,
        ATYPE_IPV6 => ATYPE_IPV6,
        _ => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Unsupported address type",
            ))
        }
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

    // Extract address and port
    let addr = match atype {
        ATYPE_IPV4 => IpAddress::from_u32(stream.read_u32().await?),
        ATYPE_IPV6 => IpAddress::from_u128(stream.read_u128().await?),
        ATYPE_DOMAIN_NAME => {
            let mut buf = Vec::with_capacity(addr_size);
            stream.read_exact(&mut buf[..addr_size]).await?;
            IpAddress::from_vec(buf)
        }
        _ => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Unsupported address type",
            ))
        }
    };

    // Read port number
    let port = stream.read_u16().await?;

    // Read CLRF
    stream.read_u16().await?;

    let request = Request::new(hex, command, atype, addr, addr_size, port);

    debug!("Read request {}", request.request_addr_port());

    Ok(request)
}
