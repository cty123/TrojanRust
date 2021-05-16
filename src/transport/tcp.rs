use crate::application::base::{InboundHandler};
use crate::protocol::common::handler::Handler;
use crate::protocol::socks5::stream::Socks5Stream;

use tokio::net::TcpStream;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio_rustls::TlsAcceptor;
use tokio_rustls::server::TlsStream;

use std::io::Error;
use futures::future;
use log::{info, warn};

pub async fn dispatch<IO>(socket: IO, tls_acceptor: TlsAcceptor, mode: &str) -> Result<(), Error>
    where
        IO: AsyncRead + AsyncWrite + Unpin
{
    // let mut stream = match tls_acceptor.accept(socket).await {
    //     Ok(stream) => stream,
    //     Err(e) => return Err(e)
    // };

    let mut stream = Socks5Stream::new(socket, [(8080u16>>8) as u8, 8080u16 as u8]);

    let request = stream.handshake().await?;

    // Starts transport process
    let mut outbound = match TcpStream::connect(
        request.request_addr_port()).await {
        Ok(s) => s,
        Err(e) => return Err(e)
    };

    info!("Established TCP connection to {}", request.request_addr_port());

    let (mut source_read, mut source_write) = tokio::io::split(stream);
    let (mut target_read, mut target_write) = outbound.split();

    match future::join(tokio::io::copy(&mut source_read, &mut target_write),
                       tokio::io::copy(&mut target_read, &mut source_write))
        .await {
        (Err(e), _) | (_, Err(e)) => Err(e.to_string()),
        _ => Ok(()),
    };

    // let (inbound, outbound) = match mode.as_str() {
    //     "client" => {
    //         //let inbound_handler = protocol::socks5::handler::Socks5Handler::new(socket);
    //         // let inbound = InboundHandler::new(Box::new(handler));
    //         return Err(String::new())
    //     }
    //     "server" => {
    //         return Err(String::new())
    //     },
    //     "socks5" =>  {
    //         // let inbound_handler = protocol::socks5::handler::Socks5Handler::new(socket);
    //         // let inbound = InboundHandler::new(Box::new(inbound_handler));
    //         // (inbound_handler, ())
    //         return Err(String::new())
    //     }
    //     _ => { return Err(String::new())}
    // };

    Ok(())
}
