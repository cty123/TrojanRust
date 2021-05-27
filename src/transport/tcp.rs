use log::{info, warn};

use std::io;
use std::io::Error;
use std::sync::Arc;

use tokio::net::TcpStream;
use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt};
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;
use tokio_rustls::rustls::ClientConfig;
use tokio_rustls::webpki::DNSNameRef;

use crate::protocol::vless::outbound::VlessOutboundStream;
use crate::protocol::vless::base::Request;
use crate::protocol::vless::inbound::VlessInboundStream;
use crate::protocol::socks5::inbound::Socks5InboundStream;
use crate::protocol::direct::stream::DirectStream;
use crate::protocol::trojan::inbound::TrojanInboundStream;

pub async fn dispatch<IO>(mut inbound_stream: IO, mode: &str) -> Result<(), Error>
    where
        IO: AsyncRead + AsyncWrite + Unpin
{
    // if mode == "server" {
    // Handle server subroutine
    let mut inbound_stream = TrojanInboundStream::new(inbound_stream);
    // let mut inbound_stream = Socks5InboundStream::new(inbound_stream, [(8080u16 >> 8) as u8, 8080u16 as u8]);
    let request = inbound_stream.handshake().await?;


    // let mut buf = [2; 1024];
    // inbound_stream.read(&mut buf).await?;

    // info!("Read bytes {:?}", buf);
    // let request = inbound_stream.read_request().await?;
    //
    let mut connection = dial(request.request_addr_port()).await?;
    let outbound_stream = DirectStream::new(connection, false);

    let (mut source_read, mut source_write) = tokio::io::split(inbound_stream);
    let (mut target_read, mut target_write) = tokio::io::split(outbound_stream);

    match futures::future::join(tokio::io::copy(&mut source_read, &mut target_write),
                       tokio::io::copy(&mut target_read, &mut source_write))
        .await {
        (Err(e), _) | (_, Err(e)) => Err(e.to_string()),
        _ => Ok(()),
    };
    // } else {
    //     let mut inbound_stream = Socks5Stream::new(inbound_stream, [(8080u16 >> 8) as u8, 8080u16 as u8]);
    //     let request = inbound_stream.handshake().await?;
    //
    //     let vless_request = request.to_vless_request();
    //
    //     info!("Dialing remote at {}", request.request_addr_port());
    //     let mut connection = match dial("10.0.0.6:8081".parse().unwrap()).await {
    //         Ok(c) => c,
    //         Err(e) => {
    //             warn!("Failed to establish connection to destination");
    //             return Err(e);
    //         }
    //     };
    //     let mut outbound_stream = VlessOutboundStream::new(connection, vless_request);
    //     // outbound_stream.write_request().await?;
    //
    //     let (mut source_read, mut source_write) = tokio::io::split(inbound_stream);
    //     let (mut target_read, mut target_write) = tokio::io::split(outbound_stream);
    //
    //     match future::join(tokio::io::copy(&mut source_read, &mut target_write),
    //                        tokio::io::copy(&mut target_read, &mut source_write))
    //         .await {
    //         (Err(e), _) | (_, Err(e)) => Err(e.to_string()),
    //         _ => Ok(()),
    //     };
    // }

    // let mut inbound_stream = Socks5Stream::new(inbound_stream, [(8080u16 >> 8) as u8, 8080u16 as u8]);
    // let request = inbound_stream.handshake().await?;
    //
    // let mut stream = dial_tls(request.request_addr_port()).await.unwrap();

    // let outbound_stream = VlessOutboundStream::new
    // let outbound_stream = DirectStream::new(stream, false);

    // // Starts transport process
    // let mut outbound = match TcpStream::connect(
    //     request.request_addr_port()).await {
    //     Ok(s) => s,
    //     Err(e) => return Err(e)
    // };

    // info!("Established TCP connection to {}", request.request_addr_port());

    // let (mut source_read, mut source_write) = tokio::io::split(inbound_stream);
    // let (mut target_read, mut target_write) = outbound.split();

    // match future::join(tokio::io::copy(&mut source_read, &mut target_write),
    //                    tokio::io::copy(&mut target_read, &mut source_write))
    //     .await {
    //     (Err(e), _) | (_, Err(e)) => Err(e.to_string()),
    //     _ => Ok(()),
    // };

    Ok(())
}

pub async fn dial(addr_port: String) -> io::Result<TcpStream> {
    return TcpStream::connect(addr_port).await;
}

pub async fn dial_tls(addr_port: String) -> io::Result<TlsStream<TcpStream>> {
    let mut config = ClientConfig::new();
    config
        .root_store
        .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);

    let connector = TlsConnector::from(Arc::new(config));
    let dnsname = DNSNameRef::try_from_ascii_str("www.rust-lang.org").unwrap();

    let stream = TcpStream::connect(addr_port).await?;
    let stream = connector.connect(dnsname, stream).await?;

    Ok(stream)
}