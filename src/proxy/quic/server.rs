use crate::{
    config::base::InboundConfig,
    config::{base::OutboundConfig, tls::make_server_config},
    protocol::trojan::parse,
};
use futures::StreamExt;
use quinn;
use std::{io::Result, net::SocketAddr};
use std::net::ToSocketAddrs;
use tokio::net::TcpStream;

pub async fn start(
    inbound_config: &'static InboundConfig,
    _outboud_config: &'static OutboundConfig,
) -> Result<()> {
    let address = (inbound_config.address.clone(), inbound_config.port)
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();

    // Build config for accepting QUIC connection
    // TODO: Avoid using unwrap
    let server_crypto = make_server_config(&inbound_config.tls.clone().unwrap()).unwrap();

    let config = quinn::ServerConfig::with_crypto(server_crypto);

    // Create QUIC server socket
    let (_endpoint, mut socket) = quinn::Endpoint::server(config, address).unwrap();

    // Start accept loop to handle incomming QUIC connections
    while let Some(conn) = socket.next().await {
        // Handle the new connection
        tokio::spawn(async move {
            // Establish QUIC connection with handshake
            let quinn::NewConnection {
                connection: _,
                mut bi_streams,
                ..
            } = match conn.await {
                Ok(c) => c,
                Err(_) => return,
            };

            // Extract reader stream and writer stream from the established connection
            let (mut client_writer, mut client_reader) = match bi_streams.next().await {
                Some(stream) => stream.unwrap(),
                None => return,
            };

            // Read proxy request from the client stream
            let request = parse(&mut client_reader).await.unwrap().into_request();

            // Connect to remote server
            let addr_port: SocketAddr = request.addr_port.into();
            let outbound_connection = TcpStream::connect(addr_port)
                .await
                .unwrap();

            // Transport data between client and remote server
            let (mut server_reader, mut server_writer) = tokio::io::split(outbound_connection);

            tokio::select!(
                _ = tokio::io::copy(&mut client_reader, &mut server_writer) => (),
                _ = tokio::io::copy(&mut server_reader, &mut client_writer) => ()
            );
        });
    }

    Ok(())
}
