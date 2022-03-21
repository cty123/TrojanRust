use crate::config::base::{InboundConfig, OutboundConfig};
use crate::protocol::common::stream::StandardTcpStream;
use crate::transport::grpc::proxy_service_server::ProxyServiceServer;
use crate::transport::grpc::GrpcPacket;
use crate::transport::grpc::GrpcService;
use bytes::BytesMut;
use std::io::{self, Error, ErrorKind};
use std::net::ToSocketAddrs;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::sync::mpsc::Sender;
use tonic::transport::Server;
use tonic::{Status, Streaming};

pub async fn start(inbound_config: InboundConfig, outbound_config: OutboundConfig) {
    let address = (inbound_config.address.clone(), inbound_config.port)
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();

    let mut server = Server::builder();
    server
        .add_service(ProxyServiceServer::new(GrpcService::new()))
        .serve(address)
        .await
        .unwrap();
}

pub async fn handle_server_data<T: AsyncRead + AsyncWrite + Unpin>(
    mut client_reader: Streaming<GrpcPacket>,
    mut server_writer: WriteHalf<StandardTcpStream<T>>,
) -> io::Result<()> {
    loop {
        let data = match client_reader.message().await {
            Ok(res) => match res {
                Some(packet) => packet,
                None => return Ok(()),
            },
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::ConnectionReset,
                    "failed to read incoming GRPC message",
                ));
            }
        };

        server_writer.write_all(&data.datagram.unwrap()).await?;
    }
}

pub async fn handle_client_data<T: AsyncRead + AsyncWrite + Unpin>(
    client_writer: Sender<Result<GrpcPacket, Status>>,
    mut server_reader: ReadHalf<StandardTcpStream<T>>,
) -> io::Result<()> {
    loop {
        let mut buf = BytesMut::with_capacity(4096);
        let n = server_reader.read_buf(&mut buf).await?;

        if n == 0 {
            return Ok(());
        }

        match client_writer
            .send(Ok(GrpcPacket {
                packet_type: 0,
                trojan: None,
                datagram: Some(buf.to_vec()),
            }))
            .await
        {
            Ok(_) => continue,
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::ConnectionRefused,
                    "failed to write to back GRPC",
                ))
            }
        }
    }
}
