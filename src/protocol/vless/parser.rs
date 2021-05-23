use tokio::io::{AsyncWriteExt, AsyncRead, AsyncReadExt};

use std::io::{Result, Error, ErrorKind};
use std::convert::TryInto;

use crate::protocol::vless::base::VERSION;
use crate::protocol::vless::base::Request;
use crate::protocol::common::command::{CONNECT, BIND, UDP_ASSOCIATE};
use crate::protocol::common::addr::{ATYPE_IPV4, ATYPE_DOMAIN_NAME, ATYPE_IPV6, IPV4_SIZE, IPV6_SIZE};

const REQUEST_HEADER_SIZE: usize = 1 + 16 + 1 + 2 + 1;
const INDEX_VERSION: usize = 0;
const INDEX_UUID: usize = 1;
const INDEX_COMMAND: usize = INDEX_UUID + 16;
const INDEX_PORT: usize = INDEX_COMMAND + 1;
const INDEX_ATYPE: usize = INDEX_PORT + 2;

pub async fn parse<IO>(mut stream: IO) -> Result<Request>
    where
        IO: AsyncReadExt + AsyncWriteExt + Unpin
{
    // Read vless header message
    let mut buf = [0; REQUEST_HEADER_SIZE];
    stream.read_exact(&mut buf).await?;

    let version = match buf[INDEX_VERSION] {
        VERSION => VERSION,
        _ => return Err(Error::new(ErrorKind::InvalidInput, "Incorrect version number")),
    };

    // Extract the UUID
    let uuid: [u8; 16] = buf[1..17].try_into().expect("Buffer has incorrect size");

    let command = match buf[INDEX_COMMAND] {
        CONNECT => CONNECT,
        BIND => BIND,
        UDP_ASSOCIATE => UDP_ASSOCIATE,
        _ => return Err(Error::new(ErrorKind::InvalidInput, "Unsupported command"))
    };

    // Extract the port number
    let port = [buf[INDEX_PORT], buf[INDEX_PORT + 1]];

    let atype = match buf[INDEX_ATYPE] {
        ATYPE_IPV4 => ATYPE_IPV4,
        ATYPE_DOMAIN_NAME => ATYPE_DOMAIN_NAME,
        ATYPE_IPV6 => ATYPE_IPV6,
        _ => return Err(Error::new(ErrorKind::InvalidInput, "Unsupported address type"))
    };

    // Read destination address
    let mut buf = [0; 16];
    match atype {
        1 => {
            stream.read_exact(&mut buf[0..IPV4_SIZE]).await?;
        }
        4 => {
            stream.read_exact(&mut buf[0..IPV6_SIZE]).await?;
        }
        _ => return Err(Error::new(ErrorKind::InvalidInput, "Failed to parse request data")),
    };

    let request = Request::new(version, uuid, command, port, atype, buf);

    return Ok(request);
}
