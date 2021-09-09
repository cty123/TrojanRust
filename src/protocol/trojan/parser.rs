use std::io::Result;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::protocol::common::addr::{IpAddress, IPV4_SIZE, IPV6_SIZE};
use crate::protocol::common::atype::Atype;
use crate::protocol::common::command::Command;
use crate::protocol::trojan::base::{Request, HEX_SIZE};

pub async fn parse<IO>(mut stream: IO) -> Result<Request>
where
    IO: AsyncReadExt + AsyncWriteExt + Unpin,
{
    // Read hex value for authentication
    let mut hex = [0u8; HEX_SIZE];
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
            let size = usize::from(stream.read_u8().await?);
            let mut buf = vec![0u8; size];
            stream.read_exact(&mut buf).await?;
            (size, IpAddress::from_vec(buf))
        }
    };

    // Read port number
    let port = stream.read_u16().await?;

    // Read CLRF
    stream.read_u16().await?;

    Ok(Request::new(hex, command, atype, addr, port))
}
