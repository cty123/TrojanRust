use std::io::Result;
use std::io::{Error, ErrorKind};

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};

use crate::protocol::common::addr::{IpAddress, IPV4_SIZE, IPV6_SIZE};
use crate::protocol::common::atype::Atype;
use crate::protocol::common::command::Command;
use crate::protocol::socks5::base::{Request, VERSION};

pub async fn parse<IO>(mut stream: IO) -> Result<Request>
where
    IO: AsyncRead + AsyncWrite + Unpin,
{
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
    let (_, addr) = match atype {
        Atype::IPv4 => (IPV4_SIZE, IpAddress::from_u32(stream.read_u32().await?)),
        Atype::IPv6 => (IPV6_SIZE, IpAddress::from_u128(stream.read_u128().await?)),
        Atype::DomainName => {
            let size = usize::from(stream.read_u8().await?);
            let mut buf = Vec::with_capacity(size);
            stream.read_exact(&mut buf).await?;
            (size, IpAddress::from_vec(buf))
        }
    };

    // Read port number
    let port = stream.read_u16().await?;

    let request = Request::new(version, command, rsv, atype, port, addr);

    return Ok(request);
}
