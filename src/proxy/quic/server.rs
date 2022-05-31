use futures::StreamExt;
use log::info;
use quinn;
use std::io::Result;
use std::net::ToSocketAddrs;
use tokio::net::TcpStream;

use crate::{
    config::base::InboundConfig,
    config::{base::OutboundConfig, tls::make_server_config},
    protocol::trojan::parse,
};

pub async fn start(inbound_config: InboundConfig, _outboud_config: OutboundConfig) -> Result<()> {
    let address = (inbound_config.address.clone(), inbound_config.port)
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();

    // Build config for accepting QUIC connection
    // TODO: Avoid using unwrap
    let server_crypto = make_server_config(&inbound_config.tls.unwrap()).unwrap();

    let config = quinn::ServerConfig::with_crypto(server_crypto);

    // Create QUIC server socket
    let (_endpoint, mut socket) = quinn::Endpoint::server(config, address).unwrap();

    // Start accept loop to handle incomming QUIC connections
    while let Some(conn) = socket.next().await {
        // Handle the new connection
        tokio::spawn(async move {
            // Establish QUIC connection with handshake
            let quinn::NewConnection {
                connection,
                mut bi_streams,
                ..
            } = conn.await.unwrap();

            // Extract reader stream and writer stream from the established connection
            let (mut client_writer, mut client_reader) = match bi_streams.next().await {
                Some(stream) => stream.unwrap(),
                None => return,
            };

            // Read proxy request from the client stream
            // TODO: Support other proxy protocols
            let request = parse(&mut client_reader).await.unwrap().inbound_request();

            // Connect to remote server
            let outbound_connection = TcpStream::connect(request.into_destination_address())
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
