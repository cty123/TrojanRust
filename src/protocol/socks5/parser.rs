use log::debug;

use std::io::{Error, ErrorKind};
use std::io::Result;

use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt};

use crate::protocol::socks5::base::Request;
use crate::protocol::common::command::Command;
use crate::protocol::common::addr::{AType, DomainName, IPv4Addr, IPv6Addr};

pub async fn parse<IO>(mut stream: IO) -> Result<Request>
    where
        IO: AsyncRead + AsyncWrite + Unpin
{
    let mut buf = [0; 128];

    stream.read(&mut buf).await?;

    let version = match buf[0] {
        5 => 5,
        _ => return Err(Error::new(ErrorKind::InvalidInput, "Failed to parse request data")),
    };

    let command = buf[1];

    let rsv = buf[2];

    let atype = buf[3];

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
