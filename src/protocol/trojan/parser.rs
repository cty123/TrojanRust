// use log::debug;

// use std::io::{Result, Error, ErrorKind};

// use tokio::io::{AsyncWriteExt, AsyncReadExt};

// use crate::protocol::trojan::base::{Request, UdpRequest};
// use crate::protocol::common::command::{CONNECT, UDP};
// use crate::protocol::common::addr::{ATYPE_IPV4, ATYPE_IPV6, ATYPE_DOMAIN_NAME, IPV4_SIZE, IPV6_SIZE};

// pub async fn parse<IO>(mut stream: IO) -> Result<Request>
//     where
//         IO: AsyncReadExt + AsyncWriteExt + Unpin {

//     // Read hex value for authentication
//     let mut hex = [0; 56];
//     stream.read_exact(&mut hex).await?;

//     let mut crlf = [0; 2];
//     stream.read_exact(&mut crlf).await?;

//     // Extract command
//     let command = match stream.read_u8().await? {
//         CONNECT => CONNECT,
//         UDP_ASSOCIATE => UDP_ASSOCIATE,
//         _ => return Err(Error::new(ErrorKind::InvalidInput, "Unsupported command"))
//     };

//     // Extract address type
//     let atype = match stream.read_u8().await? {
//         ATYPE_IPV4 => ATYPE_IPV4,
//         ATYPE_DOMAIN_NAME => ATYPE_DOMAIN_NAME,
//         ATYPE_IPV6 => ATYPE_IPV6,
//         _ => return Err(Error::new(ErrorKind::InvalidInput, "Unsupported address type"))
//     };

//     // Extract address and port
//     let mut addr = [0; 256];
//     let addr_size = match atype {
//         ATYPE_IPV4 => {
//             stream.read_exact(&mut addr[0..IPV4_SIZE]).await?;
//             IPV4_SIZE
//         }
//         ATYPE_IPV6 => {
//             stream.read_exact(&mut addr[0..IPV6_SIZE]).await?;
//             IPV6_SIZE
//         }
//         ATYPE_DOMAIN_NAME => {
//             let addr_size = usize::from(stream.read_u8().await?);
//             stream.read_exact(&mut addr[0..addr_size]).await?;
//             addr_size
//         }
//         _ => return Err(Error::new(ErrorKind::InvalidInput, "Unsupported address type"))
//     };

//     // Read port number
//     let mut port = stream.read_u16().await?;

//     let request = Request::new(hex, command, atype, addr, addr_size, port);

//     let mut crlf = [0; 2];
//     stream.read_exact(&mut crlf).await?;

//     debug!("Read request {}", request.request_addr_port());

//     Ok(request)
// }

// pub async fn parse_udp<IO>(mut stream: IO) -> Result<UdpRequest>
//     where
//         IO: AsyncReadExt + AsyncWriteExt + Unpin {

//     // Extract address type
//     let atype = stream.read_u8().await?;

//     let mut addr = [0; 256];
//     let addr_size = match atype {
//         ATYPE_IPV4 => {
//             stream.read_exact(&mut addr[0..IPV4_SIZE]).await?;
//             IPV4_SIZE
//         }
//         ATYPE_IPV6 => {
//             stream.read_exact(&mut addr[0..IPV6_SIZE]).await?;
//             IPV6_SIZE
//         }
//         ATYPE_DOMAIN_NAME => {
//             let addr_size = usize::from(stream.read_u8().await?);
//             stream.read_exact(&mut addr[0..addr_size]).await?;
//             addr_size
//         }
//         _ => return Err(Error::new(ErrorKind::InvalidInput, "Unsupported address type"))
//     };


//     // Extract port number
//     let mut port = stream.read_u16().await?;

//     // Read payload length
//     let payload_size = usize::from(stream.read_u16().await?);

//     // Read trailing CRLF
//     let mut crlf = [0; 2];
//     stream.read_exact(&mut crlf).await?;

//     // Read payload
//     let mut payload = [0; 2048];
//     stream.read_exact(&mut payload[0..payload_size]).await?;

//     let request = UdpRequest::new(atype, addr, addr_size, port, payload, payload_size);

//     Ok(request)
// }