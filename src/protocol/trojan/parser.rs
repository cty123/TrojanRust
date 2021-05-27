use log::debug;

use std::convert::TryInto;
use std::fs::read;
use std::io::{Result, Error, ErrorKind};

use tokio::io::{AsyncWriteExt, AsyncReadExt};

use crate::protocol::trojan::base::Request;
use crate::protocol::common::command::{CONNECT, UDP_ASSOCIATE};
use crate::protocol::common::addr::{ATYPE_IPV4, ATYPE_IPV6, ATYPE_DOMAIN_NAME, IPV4_SIZE, IPV6_SIZE};

pub async fn parse<IO>(mut stream: IO) -> Result<Request>
    where
        IO: AsyncReadExt + AsyncWriteExt + Unpin {

    // Read hex value for authentication
    let mut hex = [0; 56];
    stream.read_exact(&mut hex).await?;

    let mut crlf = [0; 2];
    stream.read_exact(&mut crlf).await?;

    // Extract command
    let command = match stream.read_u8().await? {
        CONNECT => CONNECT,
        UDP_ASSOCIATE => UDP_ASSOCIATE,
        _ => return Err(Error::new(ErrorKind::InvalidInput, "Unsupported command"))
    };

    // Extract address type
    let atype = match stream.read_u8().await? {
        ATYPE_IPV4 => ATYPE_IPV4,
        ATYPE_DOMAIN_NAME => ATYPE_DOMAIN_NAME,
        ATYPE_IPV6 => ATYPE_IPV6,
        _ => return Err(Error::new(ErrorKind::InvalidInput, "Unsupported address type"))
    };

    // Extract address and port
    let mut addr = [0; 256];
    let addr_size = match atype {
        ATYPE_IPV4 => {
            let mut buf = [0; 4];
            stream.read_exact(&mut buf).await?;
            addr[0..IPV4_SIZE].copy_from_slice(&buf);
            IPV4_SIZE
        }
        ATYPE_IPV6 => {
            let mut buf = [0; 16];
            stream.read_exact(&mut buf).await?;
            addr[0..IPV6_SIZE].copy_from_slice(&buf);
            IPV6_SIZE
        },
        ATYPE_DOMAIN_NAME => {
            let addr_size = usize::from(stream.read_u8().await?);
            let mut buf = vec![0; addr_size];
            stream.read_exact(&mut buf).await?;
            addr[0..addr_size].copy_from_slice(&buf);
            addr_size
        }
        _ => return Err(Error::new(ErrorKind::InvalidInput, "Unsupported address type"))
    };

    let mut port = [0; 2];
    stream.read_exact(&mut port).await?;

    let request = Request::new(hex, command, atype, addr, addr_size, port);

    let mut crlf = [0; 2];
    stream.read_exact(&mut crlf).await?;

    debug!("Read request {}", request.request_addr_port());

    //
    // if command == UDP_ASSOCIATE {
    //     return read_udp(stream).await;
    // }

    Ok(request)
}

// async fn read_udp<IO>(mut stream: IO) -> Result<Request>
//     where
//         IO: AsyncReadExt + AsyncWriteExt + Unpin {
//
//     let atype = stream.read_u8().await?;
//
//     let mut addr = [0; 16];
//     let mut port = [0; 2];
//
//     match atype {
//         1 => {
//             let mut buf = [0; 4 + 2 + 2 + 2];
//             stream.read_exact(&mut buf).await?;
//             addr[0..4].copy_from_slice(&buf[0..4]);
//             port.copy_from_slice(&buf[4..6]);
//         }
//         4 => {
//             let mut buf = [0; 16 + 2 + 2 + 2];
//             stream.read_exact(&mut buf).await?;
//             addr.copy_from_slice(&buf[0..16]);
//             port.copy_from_slice(&buf[16..18]);
//         }
//         _ => {}
//     }
//
//     let request = Request::new([0; 56], UDP_ASSOCIATE, atype, addr, port);
//
//     Ok(request)
// }