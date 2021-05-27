use log::debug;

use std::io::{Error, ErrorKind};
use std::io::Result;

use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt};

use crate::protocol::socks5::base::Request;
use crate::protocol::common::command::{CONNECT, BIND, UDP_ASSOCIATE};
use crate::protocol::common::addr::{ATYPE_IPV4, ATYPE_DOMAIN_NAME, ATYPE_IPV6, IPV4_SIZE, IPV6_SIZE};

const VERSION: u8 = 5;

pub async fn parse<IO>(mut stream: IO) -> Result<Request>
    where
        IO: AsyncRead + AsyncWrite + Unpin
{
    let version = match stream.read_u8().await? {
        VERSION => VERSION,
        _ => return Err(Error::new(ErrorKind::InvalidInput, "Incorrect version number")),
    };

    let command = match stream.read_u8().await? {
        CONNECT=> CONNECT,
        BIND => BIND,
        UDP_ASSOCIATE => UDP_ASSOCIATE,
        _ => return Err(Error::new(ErrorKind::InvalidInput, "Unsupported command"))
    };

    // Don't do anything about rsv
    let rsv = stream.read_u8().await?;

    let atype = match stream.read_u8().await? {
        ATYPE_IPV4 => ATYPE_IPV4,
        ATYPE_DOMAIN_NAME => ATYPE_DOMAIN_NAME,
        ATYPE_IPV6 => ATYPE_IPV6,
        _ => return Err(Error::new(ErrorKind::InvalidInput, "Unsupported address type"))
    };

    let mut addr = [0; 16];
    match atype {
        ATYPE_IPV4 => {
            let mut buf = [0; IPV4_SIZE];
            stream.read_exact(&mut buf).await?;
            addr[0..IPV4_SIZE].copy_from_slice(&buf);
        },
        ATYPE_IPV6 => {
            let mut buf = [0; IPV6_SIZE];
            stream.read_exact(&mut buf).await?;
            addr[0..IPV6_SIZE].copy_from_slice(&buf);
        }
        _ => return Err(Error::new(ErrorKind::InvalidInput, "Unsupported address type"))
    }

    let mut port = [0; 2];
    stream.read(&mut port).await?;

    let request = Request::new(version, command, rsv, atype, port, addr);

    return Ok(request);
}
