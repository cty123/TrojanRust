use log::debug;

use std::io::{Error, ErrorKind};
use std::io::Result;

use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt};

use crate::protocol::socks5::base::Request;
use crate::protocol::common::command::{CONNECT, BIND, UDP_ASSOCIATE};
use crate::protocol::common::addr::{ATYPE_IPV4, ATYPE_DOMAIN_NAME, ATYPE_IPV6};

const VERSION: u8 = 5;

pub async fn parse<IO>(mut stream: IO) -> Result<Request>
    where
        IO: AsyncRead + AsyncWrite + Unpin
{
    let mut buf = [0; 128];
    stream.read(&mut buf).await?;

    let version = match buf[0] {
        VERSION => VERSION,
        _ => return Err(Error::new(ErrorKind::InvalidInput, "Incorrect version number")),
    };

    let command = match buf[1] {
        CONNECT=> CONNECT,
        BIND => BIND,
        UDP_ASSOCIATE => UDP_ASSOCIATE,
        _ => return Err(Error::new(ErrorKind::InvalidInput, "Unsupported command"))
    };

    // Don't do anything about rsv
    let rsv = buf[2];

    let atype = match buf[3] {
        ATYPE_IPV4 => ATYPE_IPV4,
        ATYPE_DOMAIN_NAME => ATYPE_DOMAIN_NAME,
        ATYPE_IPV6 => ATYPE_IPV6,
        _ => return Err(Error::new(ErrorKind::InvalidInput, "Unsupported address type"))
    };

    let mut addr = [0; 16];
    let mut port = [0; 2];
    match atype {
        1 => {
            addr[0..4].copy_from_slice(&buf[4..8]);
            port[0..2].copy_from_slice(&buf[8..10]);
        },
        4 => {
            addr[0..16].copy_from_slice(&buf[4..20]);
            port[0..2].copy_from_slice(&buf[20..22]);
        }
        _ => {}
    }

    let request = Request::new(version, command, rsv, atype, port, addr);

    return Ok(request);
}
