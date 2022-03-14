use crate::protocol::common::stream::StandardTcpStream;
use crate::transport::grpc::GrpcPacket;
use std::io::{Error, ErrorKind, Result};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::sync::mpsc::Sender;
use tonic::Streaming;

pub async fn handle_client_data<T: AsyncRead + AsyncWrite + Unpin>(
    client_writer: &mut WriteHalf<StandardTcpStream<T>>,
    server_reader: &mut Streaming<GrpcPacket>,
) -> Result<()> {
    loop {
        let message = match server_reader.message().await {
            Ok(res) => match res {
                Some(packet) => packet,
                None => continue,
            },
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::ConnectionRefused,
                    "Failed to receive message",
                ))
            }
        };

        client_writer.write_all(&message.datagram.unwrap()).await?;
    }
}

pub async fn handle_server_data<T: AsyncRead + AsyncWrite + Unpin>(
    client_reader: &mut ReadHalf<StandardTcpStream<T>>,
    server_writer: &mut Sender<GrpcPacket>,
) -> Result<()> {
    loop {
        let mut buf = bytes::BytesMut::with_capacity(4096);
        client_reader.read_buf(&mut buf).await?;

        match server_writer
            .send(GrpcPacket {
                packet_type: 0,
                trojan: None,
                datagram: Some(buf.to_vec()),
            })
            .await
        {
            Ok(_) => continue,
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::ConnectionRefused,
                    "Failed to receive message",
                ));
            }
        }
    }
}
