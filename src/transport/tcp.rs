use log::{info, warn};

use std::io;
use std::io::Error;
use std::sync::Arc;

use tokio::net::{TcpStream, UdpSocket};
use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt};
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;
use tokio_rustls::rustls::ClientConfig;
use tokio_rustls::webpki::DNSNameRef;

use crate::protocol::socks5::inbound::Socks5InboundStream;
use crate::protocol::direct::stream::DirectStream;
// use crate::protocol::trojan::inbound::TrojanInboundStream;
use crate::infra::udp::stream::UdpStream;
use crate::protocol::common::command::UDP;
// use crate::protocol::trojan::packet::PacketTrojanOutboundStream;

pub async fn dispatch<IO>(mut inbound_stream: IO, mode: &str) -> Result<(), Error>
    where
        IO: AsyncRead + AsyncWrite + Unpin
{
    // if mode == "server" {
    // Handle server subroutine
    let mut inbound_stream = Socks5InboundStream::new(inbound_stream, 8080u16);
    let request = inbound_stream.handshake().await?;

    // if request.() == UDP {
        // info!("Handling udp connection");

        // let mut connection = PacketTrojanOutboundStream::new().await?;

        // let (mut source_read, mut source_write) = tokio::io::split(inbound_stream);
        // let (mut target_read, mut target_write) = tokio::io::split(connection);

        // match futures::future::join(tokio::io::copy(&mut source_read, &mut target_write),
        //                             tokio::io::copy(&mut target_read, &mut source_write))
        //     .await {
        //     (Err(e), _) | (_, Err(e)) => Err(e.to_string()),
        //     _ => Ok(()),
        // };
    // } else {
        info!("Dialing tcp {}", request.request_addr_port());
        let connection = TcpStream::connect(&request.request_addr_port()).await?;
        // let mut connection = dial(request.request_addr_port()).await?;
        let outbound_stream = DirectStream::new(connection, false);

        let (mut source_read, mut source_write) = tokio::io::split(inbound_stream);
        let (mut target_read, mut target_write) = tokio::io::split(outbound_stream);

        match futures::future::join(tokio::io::copy(&mut source_read, &mut target_write),
                                    tokio::io::copy(&mut target_read, &mut source_write))
            .await {
            (Err(e), _) | (_, Err(e)) => Err(e.to_string()),
            _ => Ok(()),
        };
    // }

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