use tokio::io::{AsyncWriteExt, AsyncRead, AsyncReadExt};

use std::io::{Result, Error, ErrorKind};
use std::convert::TryInto;

use crate::protocol::vless::base::Request;
use crate::protocol::common::addr::{IPv4Addr, AType, DomainName, IPv6Addr};

pub async fn parse<IO>(mut stream: IO) -> Result<Request>
    where
        IO: AsyncReadExt + AsyncWriteExt + Unpin
{
    // Read vless header message
    let mut buf = [0; 1 + 16 + 1 + 2 + 1];
    stream.read_exact(&mut buf).await?;

    // Read and validate the version number
    let version = match buf[0] {
        1 => 1,
        _ => return Err(Error::new(ErrorKind::InvalidInput, "Failed to parse request data")),
    };

    // Extract the UUID
    let uuid: [u8; 16] = buf[1..17].try_into().expect("Buffer has incorrect size");

    // TODO: Validate command
    let command = buf[17];

    // Extract the port number
    let port = [buf[18], buf[19]];

    // Extract vless command
    let atype = buf[20];

    // Read destination address
    let mut buf = [0; 16];
    match atype {
        1 => {
            stream.read_exact(&mut buf[0..4]).await?;
        },
        4 => {
            stream.read_exact(&mut buf[0..16]).await?;
        }
        _ => return Err(Error::new(ErrorKind::InvalidInput, "Failed to parse request data")),
    };

    let request = Request::new(version, uuid, command, port, atype, buf);

    return Ok(request)
}
