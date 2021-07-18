// use log::info;

// use std::task::{Context, Poll};
// use std::pin::Pin;
// use std::io::{Result, Error, ErrorKind};

// use tokio::io::{AsyncRead, AsyncWrite, ReadBuf, BufReader};

// use crate::protocol::trojan::parser;
// use crate::protocol::trojan::parser::parse;
// use crate::protocol::trojan::base::Request;

// pub struct TrojanInboundStream<IO> {
//     stream: BufReader<IO>,
// }

// impl<IO> AsyncRead for TrojanInboundStream<IO>
//     where
//         IO: AsyncRead + AsyncWrite + Unpin
// {
//     fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<Result<()>> {
//         return Pin::new(&mut self.stream).poll_read(cx, buf);
//     }
// }

// impl<IO> AsyncWrite for TrojanInboundStream<IO>
//     where
//         IO: AsyncRead + AsyncWrite + Unpin
// {
//     fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
//         info!("Write bytes {:?}", buf);
//         return Pin::new(&mut self.stream).poll_write(cx, buf);
//     }

//     fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
//         return Pin::new(&mut self.stream).poll_flush(cx);
//     }

//     fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
//         return Pin::new(&mut self.stream).poll_shutdown(cx);
//     }
// }

// impl<IO> TrojanInboundStream<IO>
//     where
//         IO: AsyncRead + AsyncWrite + Unpin
// {
//     pub fn new(stream: IO) -> TrojanInboundStream<IO> {
//         return TrojanInboundStream {
//             stream: tokio::io::BufReader::with_capacity(2048, stream),
//         };
//     }

//     pub async fn handshake(&mut self) -> Result<Request> {
//         let request = parse(&mut self.stream).await?;
//         info!("Read trojan request: {}", request.dump_request());
//         return Ok(request);
//     }
// }