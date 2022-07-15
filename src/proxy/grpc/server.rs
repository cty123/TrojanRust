use crate::config::base::{InboundConfig, OutboundConfig};
use crate::protocol::common::stream::StandardTcpStream;
use crate::transport::grpc::grpc_service_server::GrpcServiceServer;
use crate::transport::grpc::GrpcPacket;
use crate::transport::grpc::GrpcProxyService;

use log::info;
use std::io::{self, Error, ErrorKind};
use std::net::ToSocketAddrs;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::sync::mpsc::Sender;
use tonic::transport::{Identity, Server, ServerTlsConfig};
use tonic::{Status, Streaming};

const BUFFER_SIZE: usize = 4096;

pub async fn start(
    inbound_config: &'static InboundConfig,
    _outbound_config: &'static OutboundConfig,
) -> io::Result<()> {
    // Extract the address that the server should listen on
    let address = match (inbound_config.address.as_ref(), inbound_config.port)
        .to_socket_addrs()
        .unwrap()
        .next()
    {
        Some(addr) => addr,
        None => {
            return Err(Error::new(
                ErrorKind::AddrNotAvailable,
                "incorrect address in configuration",
            ))
        }
    };

    let tls_config = match &inbound_config.tls {
        Some(cfg) => {
            let cert = tokio::fs::read(cfg.cert_path.clone()).await?;
            let key = tokio::fs::read(cfg.key_path.clone()).await?;
            Some(ServerTlsConfig::new().identity(Identity::from_pem(cert, key)))
        }
        None => None,
    };

    info!("GRPC server listening on {}", address);

    // Initialize and start the GRPC server to serve GRPC requests
    return match Server::builder()
        .tls_config(tls_config.unwrap())
        .unwrap()
        .add_service(GrpcServiceServer::new(GrpcProxyService::new()))
        .serve(address)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::new(ErrorKind::AddrInUse, e)),
    };
}

pub async fn handle_server_data<T: AsyncRead + AsyncWrite + Unpin + Send>(
    mut client_reader: Streaming<GrpcPacket>,
    mut server_writer: WriteHalf<StandardTcpStream<T>>,
) -> io::Result<()> {
    loop {
        let data = match client_reader.message().await {
            Ok(res) => match res {
                Some(packet) => packet,
                None => continue,
            },
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::ConnectionReset,
                    "failed to read incoming GRPC message",
                ));
            }
        };

        match data.datagram {
            Some(vec) => server_writer.write_all(&vec).await?,
            None => continue,
        }
    }
}

pub async fn handle_client_data<T: AsyncRead + AsyncWrite + Unpin + Send>(
    client_writer: Sender<Result<GrpcPacket, Status>>,
    mut server_reader: ReadHalf<StandardTcpStream<T>>,
) -> io::Result<()> {
    loop {
        let mut buf = Vec::with_capacity(BUFFER_SIZE);
        let n = server_reader.read_buf(&mut buf).await?;

        if n == 0 {
            return Ok(());
        }

        buf.truncate(n);

        match client_writer
            .send(Ok(GrpcPacket {
                packet_type: 0,
                trojan: None,
                datagram: Some(buf),
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
